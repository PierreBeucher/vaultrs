extern crate tracing;

mod common;

use common::{VaultServer, VaultServerHelper};
use test_log::test;
use vaultrs::{kv1};
use std::collections::HashMap;

use vaultrs::api::kv::responses::GetSecretResponse;
use vaultrs::error::ClientError;

#[test]
fn test_kv1() {
    let test = common::new_test();
    test.run(|instance| async move {
        let server: VaultServer = instance.server();
        let client = server.client();

        // Mount KV v1 secret engine
        let mount = "kv_v1";
        let secret_path = "mysecret/foo";
        server.mount_secret(&client, mount, "kv").await.unwrap();

        // Create test secrets
        let expected_secret = HashMap::from([ 
            ("key1".to_string(), "value1".to_string()),
            ("key2".to_string(), "value2".to_string())
        ]); 
        kv1::set(&client, mount, &secret_path, &expected_secret).await.unwrap();

        // Read it
        let read_secret: HashMap<String, String> = kv1::get(&client, &mount, &secret_path).await.unwrap();

        println!("{:?}", read_secret);

        assert_eq!(read_secret.get("key1").unwrap(), expected_secret.get("key1").unwrap());
        assert_eq!(read_secret.get("key2").unwrap(), expected_secret.get("key2").unwrap());

        // Read it as raw value
        let read_secret_raw: GetSecretResponse = kv1::get_raw(&client, &mount, &secret_path).await.unwrap();

        println!("{:?}", read_secret_raw);

        assert_eq!(read_secret_raw.data.get("key1").unwrap(), expected_secret.get("key1").unwrap());
        assert_eq!(read_secret_raw.data.get("key2").unwrap(), expected_secret.get("key2").unwrap());

        // List secret keys
        let list_secret = kv1::list(&client, &mount, "mysecret").await.unwrap();

        println!("{:?}", list_secret);

        assert_eq!(list_secret.data.keys, vec!["foo"]);

        // Delete secret and read again and expect 404 to check deletion
        kv1::delete(&client, &mount, &secret_path).await.unwrap();

        let r = kv1::get_raw(&client, &mount, &secret_path).await;

        match r.expect_err(&format!("Expected error when reading {} after delete.", &secret_path)) {
            ClientError::APIError{code, ..} => { assert_eq!(code, 404, "Expected error code 404 for non-existing secret") },
            e  => { panic!("Expected error to be APIError with code 404, got {:?}", e) }
        };

    });
}