use std::{time::Duration, io::prelude::*, net::TcpStream, sync::mpsc::Sender};
use std::net::TcpListener;
use std::thread;
use std::collections::HashMap;
use std::sync::mpsc;

const GROUP_SIZE: usize = 15;
const CLIENT_NUMBER: i32 = 750;
const SIMULATION_CYCLES: i32 = 5;

fn main() {
    let mut group_loop_it = GROUP_SIZE;
    let mut group_counter = 1;
    let mut stream_group: Vec<TcpStream> = Vec::new();
    let mut stream_groups:Vec<(Vec<TcpStream>, i32)> = Vec::new();

    let (tx_stop_signal, rx_stop_signal) = mpsc::channel();
    let (tx_end_confirmation, rx_end_confirmation) = mpsc::channel();

    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let mut connection_count = 1;

    println!("Server initialized, awaiting for all clients to connect...");
    
    for stream in listener.incoming() {
        //Add the current connection of iteration to the group
        group_loop_it-=1;
        stream_group.push(stream.unwrap());

        if group_loop_it == 0 {
            //Create the thread with a completed connections group
            stream_groups.push((stream_group, group_counter));
            stream_group = Vec::new();
            group_loop_it = GROUP_SIZE;
            group_counter +=  1;
        }

        if connection_count == CLIENT_NUMBER {
            break;
        }
        connection_count+=1;
    }

    println!("All {} clients connected, starting simulation...", connection_count);
    thread::sleep(Duration::from_secs(3));

    println!("\nSimulation running...\n");
    for group in stream_groups{
        let tx1 = mpsc::Sender::clone(&tx_stop_signal);
        let tx2 = mpsc::Sender::clone(&tx_end_confirmation);
        thread::spawn(move || handle_group_connection(group.0, group.1,tx1,tx2));
    }

    for _ in 0..CLIENT_NUMBER{
        while !rx_stop_signal.recv().is_ok(){};
    }

    drop(rx_end_confirmation);

    for _ in 0..CLIENT_NUMBER{
        while !rx_stop_signal.recv().is_ok(){};
    }

    println!("\nSimulation ended.")

}

fn handle_group_connection(stream_group: Vec<TcpStream> ,group_id:i32, tx_stop_signal: Sender<()>, tx_end_confirmation: Sender<()>){
    let mut buffer = [0 as u8; 10];
    let mut group_data = HashMap::with_capacity(GROUP_SIZE);

    //Init phase
    //starting_id is, the id of the first client of the group
    let starting_id = (group_id * (GROUP_SIZE as i32)) - (GROUP_SIZE as i32);

    let mut it_id = starting_id;
    for mut stream in &stream_group{
        buffer = data_to_bytes( &(it_id,(0,0,GROUP_SIZE as i16)) );
        while !stream.write(&buffer).is_ok() {};
        it_id += 1;
    }

    //Simulation phase
    for it in 0..SIMULATION_CYCLES {
        
        //Receive coordinates
        for mut stream in &stream_group{
            while !stream.read(&mut buffer).is_ok() {};
            let data_received = bytes_to_data(&buffer);
            group_data.insert(data_received.0, data_received.1);
        }

        //Send coordinates
        for mut stream in &stream_group{
            for entry in &group_data{
                buffer = data_to_bytes(&(*entry.0,*entry.1));
                while !stream.write(& buffer).is_ok() {};
            }
        }

        if it != SIMULATION_CYCLES-1{
            //Send continue simulation signal
            for mut stream in &stream_group{
            buffer = data_to_bytes(&(0,(0,0,0)));
            while !stream.write(& buffer).is_ok() {};}
        }
    }

    //Wait for all threads before ending
    for _ in &stream_group{
        while !tx_stop_signal.send(()).is_ok(){};
    }
    while tx_end_confirmation.send(()).is_ok() {
        thread::sleep(Duration::from_secs(5));
    }

    //Send end simulation signal
    for mut stream in &stream_group{
        buffer = data_to_bytes(&(-1,(0,0,0)));
        while !stream.write(& buffer).is_ok() {};
    }

    //Closing phase
    for mut stream in &stream_group{
        while !stream.read(&mut buffer).is_ok() {};
        println!("Average millis ellapsed: {}", bytes_to_time(&buffer))
    }

    for _ in &stream_group{
        while !tx_stop_signal.send(()).is_ok(){};
    }

}

fn bytes_to_time(buffer: &[u8; 10]) -> u64 {
    let mut value= [0 as u8; 8];
    value.copy_from_slice(&buffer[..8]);
    u64::from_le_bytes(value)
}

fn bytes_to_data(buffer: &[u8; 10]) -> (i32,(i16,i16,i16)){
    let mut value=([0 as u8; 4],([0 as u8; 2],[0 as u8; 2],[0 as u8; 2]));
    value.0[..].copy_from_slice(&buffer[..4]);
    value.1.0[..].copy_from_slice(&buffer[4..6]);
    value.1.1[..].copy_from_slice(&buffer[6..8]);
    value.1.2[..].copy_from_slice(&buffer[8..]);
    
    (
        i32::from_le_bytes(value.0),
        (
            i16::from_le_bytes(value.1.0),
            i16::from_le_bytes(value.1.1),
            i16::from_le_bytes(value.1.2)
        )
    )
}

fn data_to_bytes(value: &(i32,(i16,i16,i16))) -> [u8; 10]{

    let mut buffer = [0 as u8; 10];

    buffer[..4].copy_from_slice(&value.0.to_le_bytes());
    buffer[4..6].copy_from_slice(&value.1.0.to_le_bytes());
    buffer[6..8].copy_from_slice(&value.1.1.to_le_bytes());
    buffer[8..].copy_from_slice(&value.1.2.to_le_bytes());

    buffer
}

/*
fn main() {
    let mut group_loop_it = GROUP_SIZE;
    let mut group_counter = 0;
    let mut stream_group: Vec<TcpStream> = Vec::new();

    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    let mut connection_count = 0;

    for stream in listener.incoming() {
        //Add the current connection of iteration to the group
        group_loop_it-=1;
        stream_group.push(stream.unwrap());

        if group_loop_it == 0 {
            //Create the thread with a completed connections group
            thread::spawn(move || handle_group_connection(stream_group, group_counter));
            stream_group = Vec::new();
            group_loop_it = GROUP_SIZE;
            group_counter +=  1;
        }

        //
        connection_count+=1;
        println!("Number of connections: {}", connection_count);
    }
}

fn handle_group_connection(stream_group: Vec<TcpStream> ,group_id:u32){
    let mut buffer = [0 as u8; 10];
    let mut group_data:[(i16,i16,i16,i32);GROUP_SIZE] = [(0, 0, 0, 0); GROUP_SIZE];

    let mut init_it:usize = (GROUP_SIZE-1).into();

    for mut stream in &stream_group{
        stream.read(&mut buffer).unwrap();
        let value = bytes_to_data(&buffer);
        group_data[init_it] = value;
        init_it -= 1;
        stream.set_nonblocking(true).unwrap();
    }
    
    loop{
        for mut stream in &stream_group{
            match stream.read(&mut buffer) {
                Ok(buffersize) => {
                    
                    let value = bytes_to_data(&buffer);

                    println!(
                        "Group {}, msg from {} => x:{} y:{} z:{} ID:{}",
                        group_id,
                        stream.peer_addr().unwrap(),
                        value.0,
                        value.1,
                        value.2,
                        value.3
                    );
                    stream.flush().unwrap();

                    
                    for client in &group_data{
                        if client.3 != value.3{
                            buffer = data_to_bytes(&client);
                            match stream.write(&buffer){
                                Ok(buffersize) => {},
                                Err(_error) => {}
                            }
                        }
                    }
                },
                Err(_error) => {
                    println!(
                        "Group {}, msg from {} => NO MESSAGE",
                        group_id,
                        stream.peer_addr().unwrap()
                    );
                }
            };
            thread::sleep(Duration::from_millis(10));
        }
    }
}
*/