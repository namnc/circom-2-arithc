pragma circom 2.1.0;

template directOutput() {
    signal output out;
    out <== 42;
}

component main = directOutput();
