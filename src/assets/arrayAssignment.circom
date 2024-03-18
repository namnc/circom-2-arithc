pragma circom 2.0.0;

template ComponentA () {
    signal input in[2][2];
    signal output out;

    out <== in[0][0] + in[0][1] + in[1][0] + in[1][1];
}

template ComponentB() {
    signal input a_in[2][2];
    signal output out;

    component a = ComponentA();
    a.in <== a_in;

    out <== a.out;
}

component main = ComponentB();
