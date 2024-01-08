use routeros_lib::RouterOsClient;

fn main() {
    let mut routeros_client = RouterOsClient::new("127.0.0.1:8080");
    routeros_client.write_api_data(vec!["/login".to_owned(), "=user=admin".to_owned()]);
    routeros_client.read_api_data();
}
