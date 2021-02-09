#![allow(unused)]
use std::error::Error;
use tonic::{transport::Server, Request, Response, Status, Code};
use crate::transactions::*;
use crate::proto::client_proto;
use crate::services::*;

// Proto generated server traits
use client_proto::session_server::{Session, SessionServer};

// Proto message structs
use client_proto::{LoginRequest, LogoutRequest, SignupRequest, LoginResponse };

pub async fn start_server(address: String) -> Result<(), Box<dyn Error>> {
    let addr = address.parse().unwrap();
    let session_server = SessionImplementation::default();
 
    println!("Session service listening on {}", addr);
 
    Server::builder()
        .add_service(SessionServer::new(session_server))
        .serve(addr)
        .await?;
 
    Ok(())
 }

#[derive(Default)]
pub struct SessionImplementation {}

#[tonic::async_trait]
impl Session for SessionImplementation {
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<LoginResponse>, Status> {
        let msg_ref = request.into_inner();
        let tx_login = login::TxLogin::new(
            &msg_ref.ident,
            &msg_ref.pwd,
            &msg_ref.app,
        );
        
        match tx_login.execute() {
            Ok(sess) => Ok(Response::new(sess)),
            Err(err) => Err(parse_error(err))
        }
    }

    async fn logout(&self, request: Request<LogoutRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();
        let tx_logout = logout::TxLogout::new(
            &msg_ref.cookie,
        );
        
        match tx_logout.execute() {
            Ok(_) => Ok(Response::new(())),
            Err(err) => Err(parse_error(err))
        }
    }

    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();
        let tx_signup = signup::TxSignup::new(
            &msg_ref.name, 
            &msg_ref.email, 
            &msg_ref.pwd,
        );
        
        match tx_signup.execute() {
            Ok(_) => Ok(Response::new(())),
            Err(err) => Err(parse_error(err))
        }
    }
}