use std::fmt;
use std::collections::HashMap;
use openssl::x509::{X509,X509Req,X509ReqBuilder,X509Name};
use openssl::rsa::Rsa;
use openssl::pkey::{PKey,Private};
use openssl::error::ErrorStack;
use openssl::asn1::Asn1Time;
use openssl::hash::MessageDigest;
use rpassword::prompt_password;
use std::fs;
use openssl::symm::Cipher;
use std::fs::File;
use std::io::Write;

use crate::config::*;
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
    let store_dir: String = match config.mode {
      Mode::Server => config.server_store_dir.clone(),
      Mode::Client => config.client_store_dir.clone(),
    };
    log::info!("Initializing key store...");
    match fs::create_dir_all(&store_dir) {
      Ok(()) => log::info!("Created store directory {}.",store_dir),
      Err(_) => log::error!("Failed to create store directory {}.",store_dir),
    }
    let mut store: Store = Store {
      config: config.clone(),
      keys: match Keys::new(&config) {
        Ok(keys) => keys,
        Err(err) => {
          return Err(format!("Failed to initialize keypair: {}",err));
        },
      },
      cache: HashMap::<String,X509>::new(),
    };
    match store.keys.init() {
      Ok(cert) => store.keys.cert = Some(cert),
      Err(err) => log::error!("Cert initialization failed: {}",err),
    }
    log::info!("Successfully initialized key store.");
    Ok(store)
  }
}

///////////// Keys

pub(crate) struct Keys {
  pub(crate) config: Config,
  pub(crate) keypair: Rsa<Private>,
  pub(crate) csr: X509Req,
  pub(crate) cert: Option<X509>,
}

impl Clone for Keys {
  fn clone(&self) -> Self {
    Keys { config: self.config.clone(), keypair: self.keypair.clone(), csr: Self::create_csr(&self.config,&self.keypair).unwrap(), cert: self.cert.clone() }
  }
}

impl fmt::Debug for Keys {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}",String::from_utf8(self.cert.clone().unwrap().to_text().unwrap_or(Vec::<u8>::new())).unwrap_or(String::new()))
  }
}

impl Keys {
  pub(crate) fn new(config: &Config) -> Result<Keys,String> {
    let name: String = match config.mode {
      Mode::Server => config.server_name.clone(),
      Mode::Client => config.client_name.clone(),
    };
    let store_dir: String = match config.mode {
      Mode::Server => config.server_store_dir.clone(),
      Mode::Client => config.client_store_dir.clone(),
    };
    let pkey_locked: String   = format!("{}{}.locked.pem",store_dir,name);
    let pkey_unlocked: String = format!("{}{}.unlocked.pem",store_dir,name);
    let csr_file: String      = format!("{}{}.csr",store_dir,name);
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
    let csr: X509Req = match fs::exists(&csr_file) {
      Ok(true) => {
        let pem: Vec<u8> = match fs::read(&csr_file) {
          Ok(pem)  => pem,
          Err(err) => {
            log::error!("Failed to read csr key {}: {}",csr_file,err);
            Vec::new()
          },
        };
        match X509Req::from_pem(&pem) {
          Ok(csr)  => csr,
          Err(err) => {
            log::error!("Failed to load csr from {} exists, so creating a new one: {}",csr_file,err);
            match Self::create_csr(&config,&keypair) {
              Ok(csr)  => csr,
              Err(err) => return Err(format!("Failed to generate certificate signing request: {}",err)),
            }
          },
        }
      },
      Ok(false) => {
        log::info!("Generating csr.");
        match Self::create_csr(&config,&keypair) {
          Ok(csr)  => csr,
          Err(err) => return Err(format!("Failed to generate certificate signing request: {}",err)),
        }
      },
      Err(err) => {
        log::error!("Failed to check if csr file {} exists, so creating a new one: {}",csr_file,err);
        match Self::create_csr(&config,&keypair) {
          Ok(csr)  => csr,
          Err(err) => return Err(format!("Failed to generate certificate signing request: {}",err)),
        }
      },
    };
    Ok(Keys {
      config: config.clone(),
      keypair: keypair.clone(),
      csr: csr,
      cert: None,
    })
  }

  fn gen_key(config: &Config) -> Option<Rsa<Private>> {
    let name: String = match config.mode {
      Mode::Server => config.server_name.clone(),
      Mode::Client => config.client_name.clone(),
    };
    let store_dir: String = match config.mode {
      Mode::Server => config.server_store_dir.clone(),
      Mode::Client => config.client_store_dir.clone(),
    };
    match Rsa::generate(16384) {
      Ok(keypair) => {
        let passphrase: String = match prompt_password("Private Key passphrase:") {
          Ok(passphrase) => passphrase,
          Err(err) => {
            log::error!("Failed to get passphrase: {}",err);
            String::new()
          },
        };
        if !passphrase.is_empty() {
          let pkey_locked: String   = format!("{}{}.locked.pem",store_dir,name);
          let cipher: Cipher = Cipher::aes_256_cbc();
          match keypair.private_key_to_pem_passphrase(cipher, passphrase.as_bytes()) {
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

  fn create_csr(config: &Config, keypair: &Rsa<Private>) -> Result<X509Req,ErrorStack> {
    let name: String = match config.mode {
      Mode::Server => config.server_name.clone(),
      Mode::Client => config.client_name.clone(),
    };
    let store_dir: String = match config.mode {
      Mode::Server => config.server_store_dir.clone(),
      Mode::Client => config.client_store_dir.clone(),
    };
    let csr_file: String = format!("{}{}.csr",store_dir,name);
    let pkey = PKey::from_rsa(keypair.clone())?;
    let mut name_builder = X509Name::builder()?;
    name_builder.append_entry_by_text("CN", name.as_str())?;
    let name = name_builder.build();
    let mut req_builder = X509ReqBuilder::new()?;
    req_builder.set_pubkey(&pkey)?;
    req_builder.set_subject_name(&name)?;
    req_builder.sign(&pkey, openssl::hash::MessageDigest::sha256())?;
    let csr: X509Req = req_builder.build();
    match csr.to_pem() {
      Ok(pem) => {
        match File::create(&csr_file) {
          Ok(mut file) => match file.write_all(&pem) {
            Ok(_) => {},
            Err(err) => log::error!("Failed to write csr to {}: {}",csr_file,err),
          },
          Err(err) => log::error!("Failed to create csr key file at {}: {}",csr_file,err),
        }
      },
      Err(err) => log::error!("Failed to write csr to {}: {}",csr_file,err),
    }
    Ok(csr)
  }

  fn init(&mut self) -> Result<X509,ErrorStack> {
    let name: String = match self.config.mode {
      Mode::Server => self.config.server_name.clone(),
      Mode::Client => self.config.client_name.clone(),
    };
    let store_dir: String = match self.config.mode {
      Mode::Server => self.config.server_store_dir.clone(),
      Mode::Client => self.config.client_store_dir.clone(),
    };
    let pkey = PKey::from_rsa(self.keypair.clone())?;
    let mut builder = X509::builder()?;
    builder.set_version(2)?;
    builder.set_pubkey(&pkey)?;
    let mut name_builder = X509Name::builder()?;
    name_builder.append_entry_by_text("CN", name.clone().as_str())?;
    let name = name_builder.build();
    builder.set_subject_name(&name)?;
    builder.set_issuer_name(&name)?;
    let not_before = Asn1Time::days_from_now(0)?;
    let not_after = Asn1Time::days_from_now(30)?;
    builder.set_not_before(&not_before)?;
    builder.set_not_after(&not_after)?;
    builder.sign(&pkey, MessageDigest::sha256())?;
    Ok(builder.build())
  }
}
