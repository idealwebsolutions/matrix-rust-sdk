use coarsetime::Clock;
use serde_json::{Value, Error};
use reqwest::{Response, Error as HttpError};
use reqwest;
use serde_json;

// arc/mutex
use std::collections::HashMap;
use std::io::Read;

/*
pub trait MatrixClient {
    fn login_with_password(&self) -> Result<String>;
}*/

/*
enum LoginType {
    Password { value: String }
}*/

pub struct MatrixClient<'a> {
    base_url: Cow<'a, str>,
    access_token: Cow<'a, str>,
    user_id: Cow<'a, str>
}

pub type MatrixBody = HashMap<String, String>;

impl<'a> MatrixClient<'a> {
    pub fn new(base_url: &str, user_id: &str, access_token: &str) -> MatrixClient {
        MatrixClient {
            base_url: base_url,
            access_token: access_token,
            user_id: user_id
        }
    }

    /// static function to login
    pub fn login_with_password(base_url: &str, user_id: &str, password: &str) {
        let login_api = format!("{}/_matrix/client/r0/login", &base_url.to_owned());
        let mut payload = MatrixBody::new();

        payload.insert("type".to_owned(), "m.login.password".to_owned());
        payload.insert("user".to_owned(), user_id.to_owned());
        payload.insert("password".to_owned(), password.to_owned());

        let response = MatrixClient::request("POST", &login_api, Some(payload)).unwrap();

        assert!(response.status().is_success());
    }

    pub fn refresh_token(&self) {
        //let refresh_token_api = format!("{}/_matrix/client/r0/tokenrefresh", &self.base_url);

    }

    /// This should go elsewhere
    /// TOOD: Needs urlencoding
    pub fn join_room(&self, room_or_alias: &str) {
        let join_api_endpoint = format!("{}/_matrix/client/r0/join/{}?access_token={}", &self.base_url, &room_or_alias, &self.access_token);
        let mut payload = MatrixBody::new();

        payload.insert("room_id".to_owned(), room_or_alias.to_owned());
        
        let response = MatrixClient::request("POST", &join_api_endpoint, Some(payload)).unwrap();

        assert!(response.status().is_success());
    }

    pub fn leave_room(&self, room_id: &str) {
        let leave_api_endpoint = format!("{}/_matrix/client/r0/rooms/{}/leave?access_token={}", &self.base_url, &room_id.to_owned(), &self.access_token);
        let mut payload = MatrixBody::new();
        
        payload.insert("roomId".to_owned(), room_id.to_owned());

        let response = MatrixClient::request("POST", &leave_api_endpoint, Some(payload)).unwrap();

        assert!(response.status().is_success());
    }

    /// Get room members
    pub fn get_room_members(&self, room_id: &str) {
        let room_members_api = format!("{}/_matrix/client/r0/rooms/{}/members?access_token={}", &self.base_url, &room_id.to_owned(), &self.access_token);

        let response = MatrixClient::request("GET", &room_members_api, None).unwrap();

        assert!(response.status().is_success());
    }

    /// Sends a message to a room
    pub fn send_message(&self, room_id: &str, message: &str, msgtype: &str) -> Result<Response, HttpError> {
        let timestamp = Clock::now_since_epoch().as_secs();
        let send_message_api = format!("{}/_matrix/client/r0/rooms/{}/send/m.room.message/{}?access_token={}", &self.base_url, &room_id.to_owned(), format!("{}", &timestamp), &self.access_token);
        let mut payload = MatrixBody::new();
        
        payload.insert("msgtype".to_owned(), msgtype.to_owned());
        payload.insert("body".to_owned(), message.to_owned());

        Ok(try!(MatrixClient::request("PUT", &send_message_api, Some(payload))))
        //println!(response.status());
    }

    /// Sends typing status
    pub fn send_typing_status(&self, room_id: &str, typing: bool, timeout: i32) -> Result<Response, HttpError> {
        let typing_api_endpoint = format!("{}/_matrix/client/r0/rooms/{}/typing/{}?access_token={}", &self.base_url, &room_id.to_owned(), &self.user_id, &self.access_token);
        let mut payload = MatrixBody::new();
        
        payload.insert("typing".to_owned(), format!("{}", typing.to_owned()));
        payload.insert("timeout".to_owned(), format!("{}", timeout.to_string()));

        Ok(try!(MatrixClient::request("PUT", &typing_api_endpoint, Some(payload))))
        // assert!(response.status().is_success());
    }

    /// Sends online update
    pub fn send_online_update(&self) -> Result<Response, HttpError> {
        let presence_update_api = format!("{}/_matrix/client/r0/presence/{}/status?access_token={}", &self.base_url, &self.user_id, &self.access_token);
        let mut payload = MatrixBody::new();

        payload.insert("presence".to_owned(), "online".to_owned());
        payload.insert("status_msg".to_owned(), "I am here.".to_owned());

        Ok(try!(MatrixClient::request("PUT", &presence_update_api, Some(payload))))
        //assert!(response.status().is_success());
    }
    
    /// Syncs
    pub fn sync(&self, since_from: &str, timeout: u64) -> Result<Value, Error> {
        let mut contents = String::new();
        let sync_api;

        if since_from.len() == 0 {
            sync_api = format!("{}/_matrix/client/r0/sync?access_token={}&timeout={}&set_presence=online&full_state=false", &self.base_url, &self.access_token, &timeout);
        } else {
            sync_api = format!("{}/_matrix/client/r0/sync?access_token={}&timeout={}&since={}&set_presence=online&full_state=false", &self.base_url, &self.access_token, &timeout, &since_from);
        }

        let mut response = MatrixClient::request("GET", &sync_api, None).unwrap();
    
        if !response.status().is_success() {
            println!("status is not ok: {}", response.status());
        }

        let _ = try!(response.read_to_string(&mut contents));
        
        if contents.len() == 0 {
            contents = "{}".to_owned();
        }

        let sync_result: Value = try!(serde_json::from_str(&contents));
        Ok(sync_result)
    }

    fn request(method: &str, url: &str, body: Option<MatrixBody>) -> Result<Response, HttpError> {
        let client = reqwest::Client::new().unwrap();
        let request;

        // https://stackoverflow.com/questions/38522870/are-nested-matches-a-bad-practice-in-idiomatic-rust
        if method == "POST" {
            request = client.post(url).json(&body.unwrap());
        } else if method == "PUT" {
            request = client.put(url).json(&body.unwrap());
        } else {
            request = client.get(url);
        }

        Ok(try!(request.send()))
    }
}

impl Clone for MatrixClient {
    fn clone(&self) -> MatrixClient {
        MatrixClient {
            base_url: self.base_url.clone(),
            access_token: self.access_token.clone(),
            user_id: self.user_id.clone()
        }
    }
}

//struct MatrixClientBuilder {}

//impl Clone for MatrixClient

// pub struct MatrixState {}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
