fn main()
{
  let cmd_line_error = "Error, expected: htma [host] [port]";
  //let cmd_line_error = "Error, expected: htma [directory] [host] [port]";
  let buffer = "Hello, world!";
  let mut args = std::os::args();
  let mut port: String;
  let mut host: String;
  let mut dir: String;

  match args.pop()
  {
    Some(arg) => port = arg,
    None => panic!(cmd_line_error)
  }
  match args.pop()
  {
    Some(arg) => host = arg,
    None => panic!(cmd_line_error)
  }
  /*
  match args.pop()
  {
    Some(arg) => dir = arg,
    None => panic!(cmd_line_error)
  }
  */

  println!("{:p}\n{}", &buffer, std::mem::size_of_val(&buffer));
  println!("listening: {} {}", host, port);
  htmacp::repl(host.as_slice(), port.as_slice(), htma::htparse);

  println!("le buffer: {}", buffer);
}

mod htmacp
{
  #![warn(experimental)]
  extern crate libc;

  use std;

  //because traits can't scope....
  use std::io::Listener;
  use std::io::Acceptor;

  fn mic_drop()
  {
    let mut err;

    err = unsafe { libc::funcs::posix88::unistd::setgid(99) };
    if (err != 0) { panic!("could not change group: {}", std::os::errno()); }

    err = unsafe { libc::funcs::posix88::unistd::setuid(99) };
    if (err != 0) { panic!("could not change user: {}", std::os::errno()); }

    err = unsafe { libc::funcs::posix88::unistd::chdir(std::ffi::CString::from_slice("/var/htma".as_bytes()).as_ptr()) };
    if (err != 0) { panic!("could not change directory to /var/htma: {}", std::os::errno()); }
  } 

  fn rep(mut connection: std::io::TcpStream, eval: fn (&str) -> String)
  {
    //let mut buffer = [0u8, ..1024];
    let mut buffer = [0];
    connection.read(&mut buffer);

    let buffer_str = match std::str::from_utf8(&mut buffer) {
        Ok(s) => s,
        Err(e) => "",
      };

    connection.write_str(eval(buffer_str).as_slice());
  }

  pub fn repl(ip: &str, port: &str, eval: fn (&str) -> String)
  {

    let listener = std::io::TcpListener::bind(format!("{}:{}", ip, port).as_slice())
      .ok()
      .expect("failed to bind");
    // change directory and drop privileges
    //mic_drop();
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
    size: usize,
    string: String,
    pointer: *const u8,
  }

  struct Memory_slice
  {
    pointer: *const u8,
    size: usize,
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

    req_mem.pointer = dma::get_memory_pointer(req_mem.string.as_slice());

    let http_str = dma::read_memory_pointer(req_mem.pointer, req_mem.size);

    add_headers(http_str.as_slice())
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

    // let maybe_num = std::from_str::<usize>(size_str.as_slice());
    let maybe_num = size_str.parse::<usize>();
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
    format!("HTTP/1.1 200 OK\nContent-Type: text/plain\nContent-Length: {}\n\n{}\n\n", body.len(), body)
  }
}

mod dma
{
  #![warn(experimental)]

  extern crate libc;

  use std;
  use std::num::Int;

  //takes an address and returns the data as a hex encoded string
  pub fn read_memory_pointer(memory_address: *const u8, memory_size: usize)
  -> String
  {
    // print what we're doing
    println!("/{:p}/{} -> ", memory_address, memory_size);


    //try to mprotect the page, mprotect will fail if we aren't allowed to access that memory
    //(as read/write/execute)
    let mprotect_result = unsafe
    {
      libc::funcs::posix88::mman::mprotect(((memory_address as u64) - ((memory_address as u64) % 4096)) as *mut libc::c_void,
      ((memory_size as u64) + ((memory_address as u64) % 4096)) as libc::size_t, 0x01 | 0x02 | 0x04 as libc::c_int)
    };

    let mut http_string = format!("Try this: /{:p}/32", &"Invalid memory address");

    if(-1 == mprotect_result)
    {
      println!("mprotect failed; errno = {}", std::os::errno());
      //http_string = format!("Try this: /{:p}/32", &"Invalid memory address");
    }
    else if(0 != memory_size)
    {
      println!("grabbing {} bytes from {:p}", memory_size, memory_address);
      //std::io::timer::sleep(std::time::duration::Duration::seconds(5));

      http_string = format!("");

      for i in std::iter::range(0, memory_size)
      {
        //get the byte
        let byte = unsafe { *(((memory_address as usize) + i) as *const u8) };
        //high byte
        http_string.push(u8_to_hex(byte >> 4));
        //low byte
        http_string.push(u8_to_hex(byte & 0x0f));
      }
    }

    http_string
  }

  pub fn get_memory_pointer(encoded_memory_address: &str)
  -> *const u8
  {
    let decoded_memory_address = hex_str_to_usize(encoded_memory_address);
    let p: *const u8 = decoded_memory_address as *const u8;

    p
  }

  // because nothing stable can do hex >.<
  pub fn hex_str_to_usize(hex_str: &str)
  -> usize
  {
    let mut ret_usize = 0;
    let mut sig = hex_str.len();
    let mut trailing_zero = false;

    for c in hex_str.chars()
    {
      ret_usize += ((hex_byte_to_u8(c) as usize) << 4*(sig-1));

      sig -= 1;
    }

    ret_usize
  }

  pub fn hex_byte_to_u8(c: char)
  -> u8
  {
    let mut ret_usize: u8 = 0;
    match c
    {
      '0' => { ret_usize = 0 },
      '1' => { ret_usize = 1 },
      '2' => { ret_usize = 2 },
      '3' => { ret_usize = 3 },
      '4' => { ret_usize = 4 },
      '5' => { ret_usize = 5 },
      '6' => { ret_usize = 6 },
      '7' => { ret_usize = 7 },
      '8' => { ret_usize = 8 },
      '9' => { ret_usize = 9 },
      'a' => { ret_usize = 10 },
      'b' => { ret_usize = 11 },
      'c' => { ret_usize = 12 },
      'd' => { ret_usize = 13 },
      'e' => { ret_usize = 14 },
      'f' => { ret_usize = 15 },
      _   => { ret_usize = 0; }, // we're 0x compatible!
    }

    ret_usize
  }
  pub fn u8_to_hex(byte: u8)
  -> char
  {
    let mut ret_char: char = 'x';
    match byte
    {
      0  => { ret_char = '0' },
      1  => { ret_char = '1' },
      2  => { ret_char = '2' },
      3  => { ret_char = '3' },
      4  => { ret_char = '4' },
      5  => { ret_char = '5' },
      6  => { ret_char = '6' },
      7  => { ret_char = '7' },
      8  => { ret_char = '8' },
      9  => { ret_char = '9' },
      10 => { ret_char = 'a' },
      11 => { ret_char = 'b' },
      12 => { ret_char = 'c' },
      13 => { ret_char = 'd' },
      14 => { ret_char = 'e' },
      15 => { ret_char = 'f' },
      _  => { ret_char = 'x' },
    }

    ret_char
  }
}
