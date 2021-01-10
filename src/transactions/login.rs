use crate::models::client::Controller as ClientController;
use crate::models::client::User;
use crate::transactions::*;
use crate::regex::*;

const ERR_PWD_NOT_MATCH: &str = "The provided password does not match with user's";

pub struct TxLogin<'a> {
    cookie: &'a str,
    ident: &'a str,
    pwd: &'a str,
}

impl<'a> TxLogin<'a> {
    pub fn new(cookie: &'a str, ident: &'a str, pwd: &'a str) -> Self {
        TxLogin{
            cookie: cookie,
            ident: ident,
            pwd: pwd,
        }
    }

    fn require_client_by_email(&self) -> Result<Box<dyn ClientController>, Status> {
        match User::find_by_email(self.ident) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::failed_precondition(msg);
                Err(status)
            }

            Ok(user) => Ok(user)
        }
    }

    fn require_client_by_name(&self) -> Result<Box<dyn ClientController>, Status> {
        match User::find_by_name(self.ident) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::failed_precondition(msg);
                Err(status)
            }

            Ok(user) => Ok(user)
        }
    }

    fn require_session(&self) ->  Result<&Box<dyn SessionController>, Status> {
        let provider = SessionProvider::get_instance();
        match provider.get_session_by_cookie(self.cookie) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::failed_precondition(msg);
                Err(status)
            }

            Ok(sess) => Ok(sess)
        }
    }

    fn require_password_match(&self, client: &Box<dyn ClientController>) -> Result<(), Status> {
        if !client.match_pwd(self.pwd.to_string()) {
            let msg = format!("{}", ERR_PWD_NOT_MATCH);
            let status = Status::failed_precondition(msg);
            return Err(status);
        }

        Ok(())
    }

    fn precondition_cookie(&self) ->  Result<&Box<dyn SessionController>, Status> {
        match match_cookie(self.cookie) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::failed_precondition(msg);
                Err(status)
            }

            Ok(_) => {
                self.require_session()
            }
        }
    }

    fn precondition_email(&self) -> Result<Box<dyn ClientController>, Status> {
        match match_email(self.ident) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::failed_precondition(msg);
                return Err(status);
            }

            Ok(_) => {
                let client = self.require_client_by_email()?;
                self.require_password_match(&client)?;
        
                Ok(client)
            }
        }
    }

    fn precondition_name(&self) -> Result<Box<dyn ClientController>, Status> {
        match match_name(self.ident) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::failed_precondition(msg);
                return Err(status);
            }

            Ok(_) => {
                let client = self.require_client_by_name()?;
                self.require_password_match(&client)?;
        
                Ok(client)
            }
        }
    }

    fn precondition_ident(&self) -> Result<Box<dyn ClientController>, Status> {
        match self.precondition_email() {
            Err(_) => {
                self.precondition_name()
            }

            Ok(client) => Ok(client)
        }
    }

    fn check_alive_session(&self, client: &Box<dyn ClientController>) -> Result<&Box<dyn SessionController>, Status> {
        let provider = SessionProvider::get_instance();
        match provider.get_session_by_email(&client.get_addr()) {
            Err(err) => {
                let msg = format!("{}", err);
                let status = Status::failed_precondition(msg);
                Err(status)
            }

            Ok(sess) => Ok(sess)
        }
    }

    pub fn execute(&self) -> Result<Response<SessionResponse>, Status> {
        println!("Got Login request from client {} ", self.ident);
        
        match self.precondition_cookie() {
            Err(_) => {
                let client = self.precondition_ident()?;
                let session: &Box<dyn SessionController>;
                match self.check_alive_session(&client) {
                    Err(_) => {
                        session = build_session(client)?;
                    }

                    Ok(sess) => {
                        session = sess;
                    }
                }

                println!("Session for client {} got cookie {}", session.get_client().get_addr(), session.get_cookie());
                session_response(&session, "")
            }

            Ok(session) => {
                println!("Session for client {} already exists", session.get_client().get_addr());
                session_response(&session, "")
            }
        }        
    }
}