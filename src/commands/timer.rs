use std::sync::{Arc, Mutex};
use tokio::{io::AsyncWriteExt, net::TcpStream};

use crate::{
    encoder::encode_resp_error_string, 
    encoder::encode_resp_integer,
    storage::Storage, 
    // StorageError might not be needed if get_ttl handles all cases via Ok(i64)
    // and we decide not to log the specific error _e for now.
};

pub async fn ttl(stream: &mut TcpStream, pure_cmd: Vec<String>, client_store: Arc<Mutex<Storage>>) {
    if pure_cmd.len() != 2 { // TTL takes exactly one argument (the key)
        stream
            .write_all(&encode_resp_error_string("ERR wrong number of arguments for 'ttl' command"))
            .await
            .unwrap();
        return;
    }

    let key = &pure_cmd[1];
    // .get_ttl() returns Result<i64, StorageError>.
    // The Ok variant contains the TTL value (-2 for not found/expired, -1 for no expiry, or >=0 for actual TTL).
    // An Err variant would mean an unexpected issue in storage logic beyond normal TTL outcomes.
    match client_store.lock().unwrap().get_ttl(key) {
        Ok(time_val) => {
            // The time_val from get_ttl is already an i64, so we pass it directly.
            // encode_resp_integer expects an i64 according to its usage with time_val.
            stream
                .write_all(&encode_resp_integer(time_val))
                .await
                .unwrap();
        }
        Err(_e) => { 
            // This case implies an unexpected error from the storage layer itself,
            // not a standard TTL outcome like "key not found" or "no expiry".
            // For example, if the storage encountered an internal issue.
            // A generic error message is appropriate here.
            stream
                .write_all(&encode_resp_error_string("ERR server error processing command"))
                .await
                .unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::Storage;
    use std::thread::sleep;
    use std::time::Duration;

    #[test]
    fn test_ttl_with_px_active_expiration() {
        let mut storage = Storage::new();
        let key = "mykey_px";
        storage.set_string_px(key.to_string(), "value".to_string(), 5000); // 5 seconds

        let ttl1_res = storage.get_ttl(key);
        assert!(ttl1_res.is_ok());
        let ttl1 = ttl1_res.unwrap();
        assert!(ttl1 > 0 && ttl1 <= 5, "TTL1 was {}", ttl1);

        sleep(Duration::from_secs(1));

        let ttl2_res = storage.get_ttl(key);
        assert!(ttl2_res.is_ok());
        let ttl2 = ttl2_res.unwrap();
        assert!(ttl2 < ttl1 && ttl2 > 0, "TTL2 was {}, TTL1 was {}", ttl2, ttl1);
        
        sleep(Duration::from_secs(2));

        let ttl3_res = storage.get_ttl(key);
        assert!(ttl3_res.is_ok());
        let ttl3 = ttl3_res.unwrap();
        assert!(ttl3 < ttl2 && ttl3 > 0, "TTL3 was {}, TTL2 was {}", ttl3, ttl2);
    }

    #[test]
    fn test_ttl_with_ex_active_expiration() {
        let mut storage = Storage::new();
        let key = "mykey_ex";
        storage.set_string_ex(key.to_string(), "value".to_string(), 2); // 2 seconds

        let ttl_res = storage.get_ttl(key);
        assert!(ttl_res.is_ok());
        let ttl = ttl_res.unwrap();
        // Depending on execution speed, it could be 2 or 1.
        assert!(ttl == 2 || ttl == 1, "TTL was {}", ttl);

        sleep(Duration::from_secs(1));
        let ttl2_res = storage.get_ttl(key);
         assert!(ttl2_res.is_ok());
        let ttl2 = ttl2_res.unwrap();
        assert!(ttl2 < ttl && ttl2 >=0, "TTL2 was {}, TTL was {}", ttl2, ttl);


    }

    #[test]
    fn test_ttl_for_key_without_expiration() {
        let mut storage = Storage::new();
        let key = "mykey_no_expiry";
        storage.set_string(key.to_string(), "value".to_string());

        let ttl_res = storage.get_ttl(key);
        assert!(ttl_res.is_ok());
        assert_eq!(ttl_res.unwrap(), -1);
    }

    #[test]
    fn test_ttl_for_non_existent_key() {
        let mut storage = Storage::new();
        let key = "non_existent_key";

        let ttl_res = storage.get_ttl(key);
        assert!(ttl_res.is_ok());
        assert_eq!(ttl_res.unwrap(), -2);
    }

    #[test]
    fn test_ttl_for_expired_key() {
        let mut storage = Storage::new();
        let key = "mykey_expired";
        storage.set_string_px(key.to_string(), "value".to_string(), 1); // 1 millisecond

        sleep(Duration::from_millis(100)); // Wait for 100ms to ensure expiration

        let ttl_res = storage.get_ttl(key);
        assert!(ttl_res.is_ok());
        assert_eq!(ttl_res.unwrap(), -2);
    }
}
