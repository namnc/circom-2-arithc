// from 0xZKML/zk-mnist

pragma circom 2.0.0;

include "./circomlib/switcher.circom";

template ArgMax (n) {
    signal input in[n];
    signal output out;

    // assert (out < n);
    signal gts[n];        // store comparators
    component switchers[n+1];  // switcher for comparing maxs
    component aswitchers[n+1]; // switcher for arg max

    signal maxs[n+1];
    signal amaxs[n+1];

    maxs[0] <== in[0];
    amaxs[0] <== 0;
    for(var i = 0; i < n; i++) {
        gts[i] <== in[i] > maxs[i]; // changed to 252 (maximum) for better compatibility
        switchers[i+1] = Switcher();
        aswitchers[i+1] = Switcher();

        switchers[i+1].sel <== gts[i];
        switchers[i+1].L <== maxs[i];
        switchers[i+1].R <== in[i];

        aswitchers[i+1].sel <== gts[i];
        aswitchers[i+1].L <== amaxs[i];
        aswitchers[i+1].R <== i;
        amaxs[i+1] <== aswitchers[i+1].outL;
        maxs[i+1] <== switchers[i+1].outL;
    }

    out <== amaxs[n];
}

component main = ArgMax(5);

/* INPUT = {
    "in":  ["2","3","1","5","4"],
    "out": "3"
} */