use std::{time::Duration, io::prelude::*, net::TcpStream, sync::mpsc::Sender};
use std::net::TcpListener;
use std::thread;
use std::collections::HashMap;
use std::sync::mpsc;

const GROUP_SIZE: usize = 15;
const CLIENT_NUMBER: i32 = 7500;
const SIMULATION_CYCLES: i32 = 5;
const CONN_TIMEOUT: u64 = 15;

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

    let mut avg_response_time:Vec<u64> = vec![];
    for _ in 0..CLIENT_NUMBER{
        loop{
            match rx_stop_signal.recv() {
                Ok(r) => {
                    avg_response_time.push(r);
                    break;
                },
                _ => ()
            }
        }
    }

    println!("\nSimulation ended. Average response time: {}", average(&avg_response_time))

}

fn handle_group_connection(stream_group: Vec<TcpStream> ,group_id:i32, tx_stop_signal: Sender<u64>, tx_end_confirmation: Sender<()>){
    let mut buffer = [0 as u8; 10];
    let mut group_data = HashMap::with_capacity(GROUP_SIZE);

    //Init phase
    //starting_id is, the id of the first client of the group
    let starting_id = (group_id * (GROUP_SIZE as i32)) - (GROUP_SIZE as i32);

    let mut it_id = starting_id;
    for mut stream in &stream_group{
        stream.set_nodelay(true).unwrap();
        stream.set_read_timeout(Some(Duration::new(CONN_TIMEOUT, 0))).unwrap();
        buffer = data_to_bytes( &(it_id,(SIMULATION_CYCLES as i16,0,GROUP_SIZE as i16)));
        while !stream.write(&buffer).is_ok() {};
        group_data.insert(it_id, (0,0,0));
        it_id += 1;
    }

    //Simulation phase
    for _ in 0..SIMULATION_CYCLES {
        
        //Receive coordinates
        for mut stream in &stream_group{
            while !stream.read(&mut buffer).is_ok(){}
            let data_received = bytes_to_data(&buffer);
            group_data.insert(data_received.0, data_received.1);
        }

        //Send coordinates
        for mut stream in &stream_group{
            for entry in &group_data{
                buffer = data_to_bytes(&(*entry.0,*entry.1));
                while !stream.write(& buffer).is_ok(){}
            }
        }
    }

    //Wait for all threads before ending
    for _ in &stream_group{
        while !tx_stop_signal.send(0 as u64).is_ok(){};
    }
    while tx_end_confirmation.send(()).is_ok() {
        thread::sleep(Duration::from_secs(5));
    }

    //Closing phase
    for mut stream in &stream_group{
        while !stream.read(&mut buffer).is_ok() {};
        let response_time = bytes_to_time(&buffer);
        //print!("\nAverage millis ellapsed: {}", response_time);
        while !tx_stop_signal.send(response_time).is_ok(){};
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

fn average(values: &Vec<u64>) -> u64 {
    values.iter().sum::<u64>() as u64 / values.len() as u64
}