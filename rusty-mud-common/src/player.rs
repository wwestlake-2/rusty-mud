use regex::Regex;
use pwhash::bcrypt;
extern crate zxcvbn;
use zxcvbn::zxcvbn;
//use serde_json::Result;
use uuid::Uuid;
use reindeer::{Db, Serialize, Deserialize, Entity, Error};
use directories::{
    UserDirs,
    BaseDirs,
    ProjectDirs
};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Email {
    Raw(String),
    Validated(String),
    Verified(String),
    Invalid(String)
}

impl Email {
    pub fn new(email: String) -> Self {
        Self::Raw(email)                 
    }
    
    pub fn validate(self) -> Self {
        let email_regex = Regex::new(r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})").unwrap();
        match self {
            Self::Raw(em) => match email_regex.is_match(&em) {
                true => Self::Validated(em),
                false => Self::Invalid(em)
            },
            _ => self
        }

    }

    /// todo: need to actually verify token
    pub fn verify(self, token: i32) -> Self {
        match self {
            Self::Validated(em) => Self::Verified(em),
            _ => self
        }
    }

    pub fn process(email: String) -> Self {
        Self::new(email).validate()
    }

}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Password {
    Hashed(String, u8, u64),
    Invalid(String, u8, u64)
}

impl Password {
    pub fn new(password: String) -> Self {
        let estimate = zxcvbn(&password, &[]).unwrap();
        if estimate.score() > 2 {
            Password::Hashed(bcrypt::hash(password).unwrap(), estimate.score(), estimate.crack_times().guesses())
        } else {
            Password::Invalid(String::from("Your password is not strong, please make it more complex"), estimate.score(), estimate.crack_times().guesses())
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PlayerRoles {
    Owner,
    Admin,
    Moderator,
    Player,
    Disabled
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerAccount {
    id: String,
    email: Email,
    password: Password,
    role: PlayerRoles
}

impl Entity for PlayerAccount {
    type Key = String;
    fn store_name() -> &'static str {
        "PlayerAccounts"
    }
    fn get_key(&self) -> &Self::Key {
        &self.id
    }
    fn set_key(&mut self, key : &Self::Key) {
        self.id = key.clone();
    } 
}

impl PlayerAccount {
    pub fn new(email: &str, password: &str, role: PlayerRoles) -> Self {
        Self { 
            id: Uuid::new_v4().to_string(), 
            email: Email::process(email.to_owned()), 
            password: Password::new(password.to_owned()), 
            role: role 
        }
    }

    pub fn init(db: &Db) -> Result<(), Error> {
        Self::register(db)
    }

    pub fn all(db: &Db) -> Result<Vec<PlayerAccount>, Error> {
        PlayerAccount::get_all(db)
    }

    pub fn store(&self, db: &Db) -> Result<(), Error> {
        self.save(db)
    }
}






#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn email_returns_validated_email_address() {
        let address = "someone@someplace.com".to_string();
        let email = Email::process(address);
        match email {
            Email::Validated(_) => assert!(true),
            Email::Raw(addr) => assert!(false, "email was still raw {}", addr),
            Email::Invalid(addr) => assert!(false, "email was invalid {}", addr),
            Email::Verified(addr) => assert!(false, "email was verified {}", addr)
        }
    }

    #[test]
    fn email_returns_invalid_email_address() {
        let address = "someone_someplace.com".to_string();
        let email = Email::process(address);
        match email {
            Email::Invalid(_) => assert!(true),
            Email::Validated(addr) => assert!(false, "email was valid {}", addr),
            Email::Raw(addr) => assert!(false, "email was still raw {}", addr),
            Email::Verified(addr) => assert!(false, "email was verified {}", addr)
        }
    }


}