use reqwest::ClientBuilder;
// use reqwest::asy

pub async fn get_payments() {
    // let config =
    let client_builder = ClientBuilder::new();
    let client = client_builder.build().unwrap();
    let url = "https://google.com";
    let response = client
        .request(reqwest::Method::GET, url)
        .send()
        .await
        .unwrap();
    // let result = request_builder.build().unwrap();
    // println!("Method {:?}", result.method());
    // println!("Url {:?}", result.url());
    // println!("Body {:?}", result.body());
    // println!("Header {:?}", result.headers());
    // println!("Version {:?}", result.version());

    println!("{:?}", response);
}
