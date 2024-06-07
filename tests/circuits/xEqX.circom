pragma circom 2.0.0;

template xEqX() {
    signal input x;
    signal output out;
    
    out <== x == x;
}

component main = xEqX();
