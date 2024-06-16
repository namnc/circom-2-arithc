pragma circom 2.1.0;

template constantSum() {
    signal output out;

    out <== 3 + 5;
}

component main = constantSum();
