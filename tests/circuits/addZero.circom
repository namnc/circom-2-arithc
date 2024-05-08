pragma circom 2.0.0;

template addZero() {
    signal input in;
    signal output out;

    out <== in + 0;
}

component main = addZero();
