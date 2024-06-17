pragma circom 2.1.0;

template componentA () {
    signal input in[2][2];
    signal output out;

    out <== in[0][0] + in[0][1] + in[1][0] + in[1][1];
}

template componentB() {
    signal input a_in[2][2];
    signal output out;

    component a = componentA();
    a.in <== a_in;

    out <== a.out;
}

component main = componentB();
