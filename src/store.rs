use std::fmt;
use std::collections::HashMap;
use openssl::x509::{X509,X509Req,X509ReqBuilder,X509Name};
use openssl::rsa::Rsa;
use openssl::pkey::{PKey,Private};
use openssl::error::ErrorStack;
use openssl::asn1::Asn1Time;
use openssl::hash::MessageDigest;

///////////// Store

#[derive(Debug, Clone)]
pub(crate) struct Store {
  pub(crate) keys: Keys,
  pub(crate) cache: HashMap<String,X509>,
}

impl Store {
  pub(crate) fn new(server_name: &String) -> Result<Store,String> {
    log::info!("Initializing key store...");
    let mut store: Store = Store {
      keys: match Keys::new(&server_name) {
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
  pub(crate) server_name: String,
  pub(crate) keypair: Rsa<Private>,
  pub(crate) csr: X509Req,
  pub(crate) cert: Option<X509>,
}

impl Clone for Keys {
  fn clone(&self) -> Self {
    Keys { server_name: self.server_name.clone(), keypair: self.keypair.clone(), csr: Self::create_csr(&self.server_name,&self.keypair).unwrap(), cert: self.cert.clone() }
  }
}

impl fmt::Debug for Keys {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "{}",String::from_utf8(self.cert.clone().unwrap().to_text().unwrap_or(Vec::<u8>::new())).unwrap_or(String::new()))
  }
}

impl Keys {
  pub(crate) fn new(server_name: &String) -> Result<Keys,String> {
    let keypair: Rsa<Private> = match Rsa::generate(16384) {
      Ok(keypair) => keypair,
      Err(err)    => {
        return Err(format!("Failed to generate keypair: {}",err));
      },
    };
    Ok(Keys {
      server_name: server_name.clone(),
      keypair: keypair.clone(),
      csr: match Self::create_csr(&server_name,&keypair) {
        Ok(csr) => csr,
        Err(err)    => {
          return Err(format!("Failed to generate certificate signing request: {}",err));
        },
      },
      cert: None,
    })
  }

  fn create_csr(server_name: &String, keypair: &Rsa<Private>) -> Result<X509Req,ErrorStack> {
    let pkey = PKey::from_rsa(keypair.clone())?;
    let mut name_builder = X509Name::builder()?;
    name_builder.append_entry_by_text("CN", server_name.as_str())?;
    let name = name_builder.build();
    let mut req_builder = X509ReqBuilder::new()?;
    req_builder.set_pubkey(&pkey)?;
    req_builder.set_subject_name(&name)?;
    req_builder.sign(&pkey, openssl::hash::MessageDigest::sha256())?;
    Ok(req_builder.build())
  }

  fn init(&mut self) -> Result<X509,ErrorStack> {
    let pkey = PKey::from_rsa(self.keypair.clone())?;
    let mut builder = X509::builder()?;
    builder.set_version(2)?;
    builder.set_pubkey(&pkey)?;
    let mut name_builder = X509Name::builder()?;
    name_builder.append_entry_by_text("CN", self.server_name.clone().as_str())?;
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
