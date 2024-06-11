pragma circom 2.1.0;

template xEqX() {
    signal input x;
    signal output out;
    
    out <== x == x;
}

component main = xEqX();
