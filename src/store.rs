use openssl::{
  asn1::Asn1Time,
  error::ErrorStack,
  hash::MessageDigest,
  pkey::{PKey,Private},
  rsa::Rsa,
  symm::Cipher,
  x509::{X509,X509Name},
};
use rpassword::prompt_password;
use std::{
  collections::HashMap,
  env::{args,Args},
  fmt,
  fs,
  fs::File,
  io::Write,
  path::Path,
};

use crate::config::*;
use crate::defaults::*;

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
    let mut args: Args = args();
    while let Some(arg) = args.next() {
      match arg.as_str() {
        "--lock"   => store.keys.lock(),
        "--unlock" => store.keys.unlock(),
        _ => {},
      }
    }
    store.refresh_cache();
    log::info!("Successfully initialized key store.");
    Ok(store)
  }

  pub(crate) fn refresh_cache(&mut self) {
    log::info!("Updating cert cache...");
    log::debug!("Certs in cache: {:?}",self.cache.keys());
    match fs::read_dir(&self.config.store_dir) {
      Ok(files) => {
        for file in files {
          match file {
            Ok(file) => {
              let path = file.path();
              if path.is_file() {
                match path.file_name() {
                  Some(filename) => {
                    match filename.to_str() {
                      Some(filename) => {
                        let filename: String = filename.to_string();
                        if filename.ends_with(".cert.pem") {
                          let mut id: String = filename.clone();
                          let len = filename.len() - ".cert.pem".len();
                          id.truncate(len);
                          log::debug!("Cert on disk: {}",id);
                          if !self.cache.contains_key(&id) {
                            match fs::read(&path) {
                              Ok(pem)  =>  {
                                match X509::from_pem(&pem) {
                                  Ok(cert)  => {
                                     log::debug!("Found cert {} on disk, loading...",id);
                                     self.cache.insert(id.clone(),cert.clone());
                                  },
                                  Err(err) => log::error!("Failed to parse cert {} from disk: {}",id,err),
                                }
                              },
                              Err(err) => log::error!("Failed to read cert {} from disk: {}",id,err),
                            };
                          }
                        }
                      },
                      None => {},
                    }
                  },
                  None => {},
                }
              }
            },
            Err(_) => {},
          }
        }
      },
      Err(_) => {},
    }
    for (id,cert) in &self.cache {
      let filename: String = format!("{}{}.cert.pem",self.config.store_dir,id);
      let path = Path::new(&filename);
      if !path.exists() {
        log::info!("Cert {} is in cache but not on disk, writing to disk.",id);
        match cert.to_pem() {
          Ok(pem) => {
            match File::create(&filename) {
              Ok(mut file) => { let _ = file.write_all(&pem); },
              Err(_) => {},
            }
          },
          Err(_) => {},
        }
      }
    }
  }

  pub(crate) fn verify(&mut self, id: &String, cert: &X509) -> bool {
    self.refresh_cache();
    let fingerprint: String = hex::encode(match cert.digest(MessageDigest::sha1()) {
      Ok(fingerprint) => fingerprint,
      Err(err) => {
        log::error!("Unable to retrieve fingerprint from cert: {}",err);
        return false
      },
    });
    log::info!("Verifying cert for {} with fingerprint {}",id,fingerprint);
    match self.cache.get(id) {
      Some(cached_cert) => {
        let cached_fingerprint: String = hex::encode(match cached_cert.digest(MessageDigest::sha1()) {
          Ok(fingerprint) => fingerprint,
          Err(err) => {
            log::error!("Unable to retrieve fingerprint from cached cert: {}",err);
            return false
          },
        });
        log::debug!("Found cert in cache for {} with fingerprint {}",id,cached_fingerprint);
        if fingerprint == cached_fingerprint {
          log::debug!("Fingerprint for {} matched previous fingerprint {}.",id,fingerprint);
          return true
        } else {
          log::error!("Fingerprint {} for {} doesn't match last connection.",fingerprint,id);
          return false
        };
      },
      None => {
        log::info!("First connection for {} with fingerprint {}, saving to cache.",id,fingerprint);
        self.cache.insert(id.clone(),cert.clone());
        self.refresh_cache();
        return true;
      },
    }
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
    let pkey_locked: String   = format!("{}{}.locked.pem",config.config_dir,config.name);
    let pkey_unlocked: String = format!("{}{}.unlocked.pem",config.config_dir,config.name);
    let cert_file: String     = format!("{}{}.cert.pem",config.config_dir,config.name);
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
                None
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
    let pkey_locked: String   = format!("{}{}.locked.pem",config.config_dir,config.name);
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
    let cert_file: String = format!("{}{}.cert.pem",config.config_dir,config.name);
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
    builder.sign(&pkey, MessageDigest::sha1())?;
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
    let pkey_unlocked: String   = format!("{}{}.unlocked.pem",self.config.config_dir,self.config.name);
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
    let pkey_unlocked: String   = format!("{}{}.unlocked.pem",self.config.config_dir,self.config.name);
    let path: &Path = Path::new(&pkey_unlocked);
    if path.exists() {
      match fs::remove_file(path) {
        Ok(())   => log::info!("Successfully locked private key."),
        Err(err) => log::error!("Failed to lock private key: {}",err),
      }
    }
  }
}
