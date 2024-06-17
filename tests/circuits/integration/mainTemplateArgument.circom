pragma circom 2.1.0;

template mainComponent (argument) {
    signal input in;
    signal output out;

    out <== in + argument;
}

component main = mainComponent(100);
