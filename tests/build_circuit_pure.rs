use circom_2_arithc::{circom::input::Input, program::build_circuit};
use vfs::{FileSystem, MemoryFS};

#[test]
fn test_build_circuit_pure() {
    let src = "
        pragma circom 2.0.0;

        // Two element sum
        template sum () {
            signal input a;
            signal input b;
            signal output out;
            
            out <== a + b;
        }
        
        component main = sum();
    ";

    let fs = MemoryFS::new();
    fs.create_dir("/src").unwrap();
    fs.create_file("/src/main.circom").unwrap().write_all(src.as_bytes()).unwrap();

    let circuit = build_circuit(&fs, &Input::new("/src/main.circom", "/output", None)).unwrap();
    let sim_circuit = circuit.build_sim_circuit().unwrap();

    let circuit_input = vec![1, 2];
    let res = sim_circuit.execute(&circuit_input).unwrap();
    assert_eq!(res, vec![3]);
}
