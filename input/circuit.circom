// from 0xZKML/zk-mnist

pragma circom 2.0.0;

template Test() {
    signal input a;
    signal input b;
    signal output c;

    c <== a * b;
}

component main = Test();