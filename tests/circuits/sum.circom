pragma circom 2.0.0;

// Two element sum
template sum () {
    signal input a;
    signal input b;
    signal output out;
    
    out <== a + b;
}

component main = sum();
