

pub struct Replic {
    id: u32,
    port: u32,
    replics, //vector referencia a la conexión con las replicas
    bank, //actor
    airline, //actor
    hotel, //actor
    is_leader: bool
}

impl Replic {
    pub fn new() -> Replic {

    }

    
}

impl  Replic{
    pub fn new() -> Replic{
        /*let mut bank = TcpStream::connect("127.0.0.1:7879").unwrap();
        let mut airline = TcpStream::connect("127.0.0.1:7880").unwrap();
        let mut hotel = TcpStream::connect("127.0.0.1:7881").unwrap();*/


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
    
    pub fn election() {
        for r in self.replics {
            r.send('ELECTIOn', self.i);
        }
    }

}