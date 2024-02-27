pragma circom 2.0.0;

template Switcher() {
    signal input sel;
    signal input L;
    signal input R;
    signal output outL;
    signal output outR;

    signal aux;

    aux <== (R-L)*sel;    // We create aux in order to have only one multiplication
    outL <==  aux + L;
    outR <== -aux + R;
}


template fc (width, height) {
    signal input in[width];
    signal input weights[height][width];
    signal input biases[height];
    signal output out[height];

    component rows[height];

    component relu[height];

    for(var index = 0; index < height; index++) {
        rows[index] = dot_product(width);
        for(var index_input = 0; index_input < width; index_input++) {
            rows[index].inputs[index_input] <== in[index_input];
            rows[index].weight_vector[index_input] <== weights[index][index_input];
        }
        rows[index].bias <== biases[index];
        relu[index] = div_relu(128, 12);
        relu[index].in <== rows[index].out;
        out[index] <== relu[index].out;
    }
}

template fc_no_relu (width, height) {
    signal input in[width];
    signal input weights[height][width];
    signal input biases[height];
    signal output out[height];

    component rows[height];

    for(var index = 0; index < height; index++) {
        rows[index] = dot_product(width);
        for(var index_input = 0; index_input < width; index_input++) {
            rows[index].inputs[index_input] <== in[index_input];
            rows[index].weight_vector[index_input] <== weights[index][index_input];
        }
        rows[index].bias <== biases[index];
        out[index] <== rows[index].out;
    }
}

template dot_product (width) {
    signal input inputs[width];
    signal input weight_vector[width];
    signal inter_accum[width];
    signal input bias;
    signal output out;

    inter_accum[0] <== inputs[0]*weight_vector[0];
    inter_accum[0]*0 === 0;

    for(var index = 1; index < width; index++) {
        inter_accum[index] <== inputs[index]*weight_vector[index] + inter_accum[index-1];
    }
    out <== inter_accum[width-1] + bias;
}

template ShiftRight(k) {
    signal input in;
    signal output out;
    out <== in;
}

template Sign() {
    signal input in;
    signal output sign;
}

template div_relu(k) {
    signal input in;
    signal output out;
    component shiftRight = ShiftRight(k);
    component sign = Sign();
    
    shiftRight.in <== in;
    sign.in <== shiftRight.out;

    component switcher = Switcher();
    switcher.sel <== sign.sign;
    switcher.L <== shiftRight.out;
    switcher.R <== 0;
    //switcher.outR*0 === 0;

    out <== switcher.outL;
}

template network() {
    var in_len = 32;
    var out_len = 100;
    signal input in[in_len];
    signal output out[out_len];

    component l0 = fc(32, 100);
    signal input w0[100][32];
    signal input b0[100];
    l0.weights <== w0;
    l0.biases <== b0;
    l0.in <== in;
    component l1 = fc(100, 200);
    signal input w1[200][100];
    signal input b1[200];
    l1.weights <== w1;
    l1.biases <== b1;
    l1.in <== l0.out;
    component l2 = fc_no_relu(200, 100);
    signal input w2[100][200];
    signal input b2[100];
    l2.weights <== w2;
    l2.biases <== b2;
    l2.in <== l1.out;
    out <== l2.out;
}

component main = network();