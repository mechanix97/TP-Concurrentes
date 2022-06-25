use mod commons;


pub struct Leader {
    replics, //vector referencia a la conexión con las replicas
    id: u32,
    bank,
    airline, 
    hotel

}

impl  Leader{
    pub fn new() -> Leader{
        let mut bank = TcpStream::connect("127.0.0.1:7879").unwrap();
        
        let msg1 = commons::Msg::Payment {id: 1, amount: 1};
        bank.write(&(serde_json::to_string(&msg1).unwrap()+"\n").as_bytes()).unwrap();
        
        
        let mut airline = TcpStream::connect("127.0.0.1:7880").unwrap();
        let mut hotel = TcpStream::connect("127.0.0.1:7881").unwrap();


        loop{
            let filename = r"input.txt";
            let file = fs::File::open(filename).expect("file not found!");
            let  buf_reader = BufReader::new(file);

            for line in buf_reader.lines() {
                println!("{}", line.unwrap());
            }
        }
    }

    pub fn broadcast(&self, msg: commons::msg){

    }


}