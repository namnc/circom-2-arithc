use circom_2_arithc::{circom::input::Input, program::build_circuit};
use circom_virtual_fs::{FileSystem, MemoryFs};

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

    let mut fs = MemoryFs::new("/home".into());
    fs.write(&"/src/main.circom".into(), src.as_bytes()).unwrap();

    let circuit = build_circuit(&mut fs, &Input::new("/src/main.circom", "/output", None)).unwrap();
    let sim_circuit = circuit.build_sim_circuit().unwrap();

    let circuit_input = vec![1, 2];
    let res = sim_circuit.execute(&circuit_input).unwrap();
    assert_eq!(res, vec![3]);
}
