use std::fmt;
use std::collections::HashMap;
use openssl::x509::{X509,X509Name};
use openssl::rsa::Rsa;
use openssl::pkey::{PKey,Private};
use openssl::error::ErrorStack;
use openssl::asn1::Asn1Time;
use openssl::hash::MessageDigest;
use openssl::symm::Cipher;
use rpassword::prompt_password;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::env::{args,Args};
use std::path::Path;

use crate::config::*;
use crate::defaults::DEFAULT_KEY_SIZE;
use crate::mode::*;

///////////// Store

#[derive(Debug, Clone)]
pub(crate) struct Store {
  pub(crate) config: Config,
  pub(crate) keys: Keys,
  pub(crate) cache: HashMap<String,X509>,
}

impl Store {
  pub(crate) fn new(config: &Config) -> Result<Store,String> {
    log::info!("Initializing key store...");
    match fs::create_dir_all(&config.store_dir) {
      Ok(()) => log::info!("Created store directory {}.",config.store_dir),
      Err(_) => log::error!("Failed to create store directory {}.",config.store_dir),
    }
    let store: Store = Store {
      config: config.clone(),
      keys: match Keys::new(&config) {
        Ok(keys) => keys,
        Err(err) => {
          return Err(format!("Failed to initialize keypair: {}",err));
        },
      },
      cache: HashMap::<String,X509>::new(),
    };
    let mut args: Args = args();
    while let Some(arg) = args.next() {
      match arg.as_str() {
        "--lock"   => store.keys.lock(),
        "--unlock" => store.keys.unlock(),
        _ => {},
      }
    }
    log::info!("Successfully initialized key store.");
    Ok(store)
  }
}

///////////// Keys

pub(crate) struct Keys {
  pub(crate) config: Config,
  pub(crate) keypair: Rsa<Private>,
  pub(crate) cert: X509,
}

impl Clone for Keys {
  fn clone(&self) -> Self {
    Keys { config: self.config.clone(), keypair: self.keypair.clone(), cert: self.cert.clone() }
  }
}

impl fmt::Debug for Keys {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}",String::from_utf8(self.cert.clone().to_text().unwrap_or(Vec::<u8>::new())).unwrap_or(String::new()))
  }
}

impl Keys {
  pub(crate) fn new(config: &Config) -> Result<Keys,String> {
    let pkey_locked: String   = format!("{}{}.locked.pem",config.store_dir,config.name);
    let pkey_unlocked: String = format!("{}{}.unlocked.pem",config.store_dir,config.name);
    let cert_file: String     = format!("{}{}.cert.pem",config.store_dir,config.name);
    let keypair: Option<Rsa<Private>> = match fs::exists(&pkey_unlocked) {
      Ok(true) => {
        let pem: Vec<u8> = match fs::read(&pkey_unlocked) {
          Ok(pem)  => pem,
          Err(err) => {
            log::error!("Failed to read unlocked key {}: {}",pkey_unlocked,err);
            Vec::new()
          },
        };
        match Rsa::private_key_from_pem(&pem) {
          Ok(key)  => Some(key),
          Err(err) => {
            log::error!("Failed to read unlocked key {}: {}",pkey_unlocked,err);
            Self::gen_key(&config)
          },
        }
      },
      Ok(false) => {
        match fs::exists(&pkey_locked) {
          Ok(true) => {
            let pem: Vec<u8> = match fs::read(&pkey_locked) {
              Ok(pem)  => pem,
              Err(err) => {
                log::error!("Failed to read unlocked key {}: {}",pkey_locked,err);
                Vec::new()
              },
            };
            let passphrase: String = match prompt_password("Private Key passphrase:") {
              Ok(passphrase) => passphrase,
              Err(err) => {
                log::error!("Failed to get passphrase: {}",err);
                String::new()
              },
            };
            match Rsa::private_key_from_pem_passphrase(&pem,&passphrase.as_bytes()) {
              Ok(key)  => Some(key),
              Err(err) => {
                log::error!("Failed to read unlocked key {}: {}",pkey_locked,err);
                Self::gen_key(&config)
              },
            }
          },
          Ok(false) => {
            Self::gen_key(&config)
          },
          Err(_) => {
            Self::gen_key(&config)
          },
        }
      },
      Err(_) => {
        Self::gen_key(&config)
      },
    };
    let keypair: Rsa<Private> = match keypair {
      Some(keypair) => keypair,
      None          => return Err(String::from("Failed to generate or retrieve private key.")),
    };
    let cert: X509 = match fs::exists(&cert_file) {
      Ok(true) => {
        let pem: Vec<u8> = match fs::read(&cert_file) {
          Ok(pem)  => pem,
          Err(err) => {
            log::error!("Failed to read cert from {}: {}",cert_file,err);
            Vec::new()
          },
        };
        match X509::from_pem(&pem) {
          Ok(cert)  => cert,
          Err(err) => {
            log::error!("Failed to load cert from {} exists, so creating a new one: {}",cert_file,err);
            match Self::create_cert(&config,&keypair) {
              Ok(cert)  => cert,
              Err(err) => return Err(format!("Failed to generate certificate: {}",err)),
            }
          },
        }
      },
      Ok(false) => {
        match Self::create_cert(&config,&keypair) {
          Ok(cert)  => cert,
          Err(err) => return Err(format!("Failed to generate certificate: {}",err)),
        }
      },
      Err(err) => {
        log::error!("Failed to check if cert file {} exists, so creating a new one: {}",cert_file,err);
        match Self::create_cert(&config,&keypair) {
          Ok(cert)  => cert,
          Err(err) => return Err(format!("Failed to generate certificate: {}",err)),
        }
      },
    };
    let keys: Keys = Keys {
      config: config.clone(),
      keypair: keypair.clone(),
      cert: cert,
    };
    Ok(keys)
  }

  fn gen_key(config: &Config) -> Option<Rsa<Private>> {
    match Rsa::generate(DEFAULT_KEY_SIZE) {
      Ok(keypair) => {
        let passphrase: String = match prompt_password("Private Key passphrase:") {
          Ok(passphrase) => passphrase,
          Err(err) => {
            log::error!("Failed to get passphrase: {}",err);
            String::new()
          },
        };
        if !passphrase.is_empty() {
          let pkey_locked: String   = format!("{}{}.locked.pem",config.store_dir,config.name);
          match keypair.private_key_to_pem_passphrase(Cipher::aes_256_cbc(), passphrase.as_bytes()) {
            Ok(pem) => {
              match File::create(&pkey_locked) {
                Ok(mut file) => match file.write_all(&pem) {
                  Ok(_) => {},
                  Err(err) => log::error!("Failed to write locked private key to {}: {}",pkey_locked,err),
                },
                Err(err) => log::error!("Failed to create locked private key file at {}: {}",pkey_locked,err),
              }
            },
            Err(err) => log::error!("Failed to convert private key to pem: {}",err),
          }
        };
        Some(keypair)
      },
      Err(err)    => {
        log::error!("Failed to generate keypair: {}",err);
        return None
      },
    }
  }

  fn create_cert(config: &Config, keypair: &Rsa<Private>) -> Result<X509,ErrorStack> {
    let cert_file: String = format!("{}{}.cert.pem",config.store_dir,config.name);
    let pkey = PKey::from_rsa(keypair.clone())?;
    let mut builder = X509::builder()?;
    builder.set_version(2)?;
    builder.set_pubkey(&pkey)?;
    let mut name_builder = X509Name::builder()?;
    name_builder.append_entry_by_text("CN", config.name.clone().as_str())?;
    let name = name_builder.build();
    builder.set_subject_name(&name)?;
    builder.set_issuer_name(&name)?;
    let not_before = Asn1Time::days_from_now(0)?;
    let not_after = Asn1Time::days_from_now(30)?;
    builder.set_not_before(&not_before)?;
    builder.set_not_after(&not_after)?;
    builder.sign(&pkey, MessageDigest::sha256())?;
    let cert: X509 = builder.build();
    match cert.to_pem() {
      Ok(pem) => {
        match File::create(&cert_file) {
          Ok(mut file) => match file.write_all(&pem) {
            Ok(_) => {},
            Err(err) => log::error!("Failed to write cert to {}: {}",cert_file,err),
          },
          Err(err) => log::error!("Failed to create cert file at {}: {}",cert_file,err),
        }
      },
      Err(err) => log::error!("Failed to write cert to {}: {}",cert_file,err),
    }
    Ok(cert)
  }

  fn unlock(&self) {
    log::info!("Unlocking private key.");
    let pkey_unlocked: String   = format!("{}{}.unlocked.pem",self.config.store_dir,self.config.name);
    match self.keypair.private_key_to_pem() {
      Ok(pem) => {
        match File::create(&pkey_unlocked) {
          Ok(mut file) => match file.write_all(&pem) {
            Ok(_) => {},
            Err(err) => log::error!("Failed to write unlocked private key to {}: {}",pkey_unlocked,err),
          },
          Err(err) => log::error!("Failed to create unlocked private key file at {}: {}",pkey_unlocked,err),
        }
      },
      Err(err) => log::error!("Failed to convert private key to pem: {}",err),
    }
    log::info!("Successfully unlocked private key.");
  }

  fn lock(&self) {
    log::info!("Locking private key.");
    let pkey_unlocked: String   = format!("{}{}.unlocked.pem",self.config.store_dir,self.config.name);
    let path: &Path = Path::new(&pkey_unlocked);
    if path.exists() {
      match fs::remove_file(path) {
        Ok(())   => log::info!("Successfully locked private key."),
        Err(err) => log::error!("Failed to lock private key: {}",err),
      }
    }
  }
}
