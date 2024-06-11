pragma circom 2.1.0;

template prefixOps() {
    signal input a;
    signal input b;
    signal input c;

    signal output negateA;

    signal output notA;
    signal output notB;
    signal output notC;

    signal output complementA;
    signal output complementB;
    signal output complementC;

    negateA <== -a;

    notA <== !a;
    notB <== !b;
    notC <== !c;

    complementA <== ~a;
    complementB <== ~b;
    complementC <== ~c;
}

component main = prefixOps();
