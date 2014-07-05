use std::rand::random;


use super::CLIENT_VERSION;


pub struct Client {
    client_rand: String,
}

impl Client {
    
    pub fn new() -> Client {
        Client {
            client_rand: format!("{:04u}{:04u}", random::<uint>() % 10000, random::<uint>() % 10000)
        }
    }

    pub fn with_client_rand(client_rand: String) -> Client {
        Client {
            client_rand: client_rand
        }
    }

    pub fn peer_id(&self) -> String {
        format!("-DA{:04u}-{:s}", CLIENT_VERSION, self.client_rand)
    }
}
