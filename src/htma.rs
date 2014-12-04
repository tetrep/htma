static mut sigsegv = false;

fn main()
{
  let buffer: &'static str = "Hello, world!";
  println!("{:p}\n{}", &buffer, std::mem::size_of_val(&buffer));

  /*
  let input = std::io::stdin().read_line()
    .ok()
    .expect("failed to read stdin");

  println!("{}", htma::htparse_raw(input.as_slice()));
  */

  println!("listening: 127.0.0.1 1066");
  htmacp::repl("127.0.0.1", "1066", htma::htparse);

  println!("le buffer: {}", buffer);
}

mod htmacp
{
  #![warn(experimental)]
  //because rust can't handle signals :(
  extern crate libc;

  use std;

  //because traits can't scope....
  use std::io::Listener;
  use std::io::Acceptor;

  extern fn handle_sigsegv(sig: i64)
  {
    if (11 == sig) { println!("totes seg fault\n"); }
    panic!("signal: {}", sig);
  }

  fn rep(mut connection: std::io::TcpStream, eval: fn (&str) -> String)
  {
    let mut buffer = [0u8, ..1024];
    connection.read(&mut buffer);

    let buffer_str = match std::str::from_utf8(&mut buffer) {
        Some(s) => s,
        None => "",
      };

    connection.write_str(eval(buffer_str).as_slice());
  }

  //#[link(name = "signal")]
  extern { fn signal(sig: i64, cb: extern fn(i64)); }

  pub fn repl(ip: &str, port: &str, eval: fn (&str) -> String)
  {
    //dat signal handle
    unsafe { signal(11, handle_sigsegv); }

    let listener = std::io::TcpListener::bind(format!("{}:{}", ip, port).as_slice())
      .ok()
      .expect("failed to bind");
    let mut acceptor = listener.listen()
      .ok()
      .expect("failed to listen");

    for connection in acceptor.incoming()
    {
      match connection
      {
        Err(e) => { println!("weeoweeoweeo: {}", e); }
        Ok(connection) => { rep(connection, eval); }
      }
    }

    drop(acceptor);
  }
}

mod htma
{
  #![warn(experimental)]

  use dma;
  use std;

  struct Memory
  {
    size: uint,
    string: String,
    pointer: *const u8,
  }

  enum URIState
  {
    URISpace,
    URIOptionalSlash,
    URISize,
    URIMemory,
  }

  /// `htparse` will take in the first line of an http request and return the specified memory
  /// as an http string
  pub fn htparse(input: &str)
  -> String
  {
    let mut req_mem = tktk_get(input);

    req_mem.pointer = dma::get_memory_pointer(req_mem.size, req_mem.string.as_slice());

    let http_str = unsafe { *(req_mem.pointer as *const &str) };

    println!("grabbing {} bytes from {}", req_mem.size, req_mem.pointer);

    add_headers(http_str)
  }

  fn tktk_get(input: &str)
  -> Memory
  {
    let mut memory = Memory { size: 0, string: "".to_string(), pointer: std::ptr::null() };
    let mut size_str = String::new();

    let mut state = URIState::URISpace;
    for c in input.chars()
    {
      match state
      {
        //find the first space (seperates verb and uri)
        URIState::URISpace => { if(' ' == c) { state = URIState::URIOptionalSlash; } },
        //consume slash if uri starts with it, otherwise treat it as the first character of the size
        URIState::URIOptionalSlash => { if('/' != c) { memory.string.push(c); } state = URIState::URIMemory },
        //get the address of memory we will be using, stopping when we hit a space
        URIState::URIMemory => { if('/' != c) { memory.string.push(c); } else { state = URIState::URISize; } },
        //get the amount of memory will we be using
        URIState::URISize => { if(' ' != c) { size_str.push(c); } else { break; } },
      }
    }

    let maybe_num = from_str(size_str.as_slice());
    match maybe_num
    {
      Some(number) => memory.size = number,
      None => memory.size = 0,
    }

    memory
  }

  fn add_headers(body: &str)
  -> String
  {
    //println!("HTTP 200 OK\nContent-Type: text/plain\nContent-Length: {}\n{}", body.len(), body)
    format!("HTTP 200 OK\nContent-Type: text/plain\nContent-Length: {}\n\n{}\n\n", body.len(), body)
  }
}

mod dma
{
  #![warn(experimental)]

  use std;
  use std::num::Int;

  pub fn get_memory_pointer(memory_size: uint, encoded_memory_address: &str)
  -> *const u8
  {
    let decoded_memory_address = hex_str_to_uint(encoded_memory_address);
    let p: *const u8 = decoded_memory_address as *const u8;

    p
  }

  // because nothing stable can do le hex >.<
  pub fn hex_str_to_uint(hex_str: &str)
  -> uint
  {
    let mut ret_uint = 0;
    let mut sig = hex_str.len()-1;
    let mut trailing_zero = false;

    for c in hex_str.chars()
    {
      match c
      {
        '0' => { },
        '1' => {ret_uint += 1*16.pow(sig);},
        '2' => {ret_uint += 2*16.pow(sig);},
        '3' => {ret_uint += 3*16.pow(sig);},
        '4' => {ret_uint += 4*16.pow(sig);},
        '5' => {ret_uint += 5*16.pow(sig);},
        '6' => {ret_uint += 6*16.pow(sig);},
        '7' => {ret_uint += 7*16.pow(sig);},
        '8' => {ret_uint += 8*16.pow(sig);},
        '9' => {ret_uint += 9*16.pow(sig);},
        'a' => {ret_uint += 10*16.pow(sig);},
        'b' => {ret_uint += 11*16.pow(sig);},
        'c' => {ret_uint += 12*16.pow(sig);},
        'd' => {ret_uint += 13*16.pow(sig);},
        'e' => {ret_uint += 14*16.pow(sig);},
        'f' => {ret_uint += 15*16.pow(sig);},
        _   => {ret_uint = 0; break;}, // srsly...
      }

      //println!("{}^{} => {}", c, sig, ret_uint);

      sig -= 1;
    }

    ret_uint
  }
}
