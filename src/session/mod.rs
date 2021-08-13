pub mod framework;
pub mod application;
pub mod domain;

lazy_static! {
    static ref REPO_PROVIDER: framework::InMemorySessionRepository = {
        framework::InMemorySessionRepository::new()
    }; 
}   

pub fn get_repository() -> Box<&'static dyn domain::SessionRepository> {
    Box::new(&*REPO_PROVIDER)
}

#[cfg(test)]
pub mod tests {
    use std::time::{SystemTime, Duration};
    use std::collections::HashMap;
    use crate::user::tests::new_user;
    use crate::metadata::domain::InnerMetadata;
    use crate::directory::tests::new_directory;
    use crate::app::tests::new_app;
    use crate::security;
    use crate::time::unix_timestamp;
    use crate::constants::settings;
    use super::domain::{Session, Token};

    pub fn new_session() -> Session {
        Session{
            sid: "testing".to_string(),
            deadline: SystemTime::now(),
            user: new_user(),
            apps: HashMap::new(),
            meta: InnerMetadata::new(),
            sandbox: HashMap::new(),
        }
    }

    #[test]
    fn session_new() {
        const TIMEOUT: Duration = Duration::from_secs(10);

        let user = new_user();
        let user_id = user.get_id();

        let before = SystemTime::now();
        let sess_arc = Session::new(user, TIMEOUT).unwrap();
        let after = SystemTime::now();
        let sess = sess_arc.read().unwrap();
        
        assert_eq!(settings::TOKEN_LEN, sess.sid.len());
        assert!(sess.deadline < after + TIMEOUT);
        assert!(sess.deadline > before + TIMEOUT);

        assert_eq!(sess.user.get_id(), user_id);
        assert_eq!(0, sess.apps.len());
        assert_eq!(0, sess.sandbox.len());
    }

    #[test]
    fn session_set_directory_ok() {
        let dir = new_directory();
        let app_id = dir.get_app();

        let mut sess = new_session();
        let before = SystemTime::now();
        sess.set_directory(dir).unwrap();
        let after = SystemTime::now();

        assert_eq!(1, sess.apps.len());
        assert!(sess.apps.get(&app_id).is_some());
        assert!(sess.meta.touch_at >= before && sess.meta.touch_at <= after);
    }

    #[test]
    fn session_set_directory_ko() {
        let dir = new_directory();

        let mut sess = new_session();
        sess.set_directory(dir).unwrap();

        let dir = new_directory();
        assert!(sess.set_directory(dir).is_err());
    }

    #[test]
    fn session_token_ok() {
        let app = new_app();
        let sess = new_session();
        let deadline = SystemTime::now() + Duration::from_secs(60);

        let before = SystemTime::now();
        let claim = Token::new(&sess, &app, deadline);
        let after = SystemTime::now();

        assert!(claim.iat >= before && claim.iat <= after);        
        assert_eq!(claim.exp, unix_timestamp(deadline));
        assert_eq!("oauth.alvidir.com", claim.iss);
        assert_eq!(sess.sid, claim.sub);
        assert_eq!(app.get_id(), claim.app);
    }

    #[test]
    #[ignore]
    fn session_token_encode() {
        dotenv::dotenv().unwrap();

        let app = new_app();
        let sess = new_session();
        let deadline = SystemTime::now() + Duration::from_secs(60);

        let before = SystemTime::now();
        let claim = Token::new(&sess, &app, deadline);
        let after = SystemTime::now();
        
        let token = security::encode_jwt(claim).unwrap();
        let claim = security::decode_jwt::<Token>(&token).unwrap();

        assert!(claim.iat >= before && claim.iat <= after);        
        assert_eq!(claim.exp, unix_timestamp(deadline));
        assert_eq!("oauth.alvidir.com", claim.iss);
        assert_eq!(sess.sid, claim.sub);
        assert_eq!(app.get_id(), claim.app);
    }

    #[test]
    #[ignore]
    fn session_token_ko() {
        dotenv::dotenv().unwrap();

        let app = new_app();
        let sess = new_session();
        let deadline = SystemTime::now() - Duration::from_secs(60);

        let claim = Token::new(&sess, &app, deadline);
        let token = security::encode_jwt(claim).unwrap();
        assert!(security::decode_jwt::<Token>(&token).is_err());
    }
}