pragma circom 2.0.0;

template MainComponent (argument) {
    signal input in;
    signal output out;

    out <== in + argument;
}

component main = MainComponent(100);
