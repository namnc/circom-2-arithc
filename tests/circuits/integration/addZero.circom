pragma circom 2.1.0;

template addZero() {
    signal input in;
    signal output out;

    out <== in + 0;
}

component main = addZero();
