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
    outR <==  R - aux;
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
        relu[index] = div_relu(12);
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
    // inter_accum[0]*0 === 0;

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
    // Structure from python example
    // self.fc1 = nn.Linear(2, 32)
    // self.fc2 = nn.Linear(32, 64)
    // self.fc3 = nn.Linear(64, 128)
    // self.fc4 = nn.Linear(128, 4)

    var in_len = 2;
    var out_len = 4;

    var l0_w = in_len;
    var l0_h = 5; 

    var l1_w = l0_h;
    var l1_h = 7;

    var l2_w = l1_h;
    var l2_h = 11;

    var l3_w = l2_h; 
    var l3_h = out_len;

    signal input in[in_len];
    signal output out[out_len];

    component l0 = fc(l0_w, l0_h);
    signal input w0[l0_h][l0_w];
    signal input b0[l0_h];
    for (var i = 0; i < l0_h; i++) {
        for (var j = 0; j < l0_w; j++) {
            l0.weights[i][j] <== w0[i][j];
        }
        l0.biases[i] <== b0[i];
    }
    // l0.weights <== w0;
    // l0.biases <== b0;
    for (var k = 0; k < in_len; k++) {
        l0.in[k] <== in[k];
    }

    component l1 = fc(l1_w, l1_h);
    signal input w1[l1_h][l1_w];
    signal input b1[l1_h];
    for (var i = 0; i < l1_h; i++) {
        for (var j = 0; j < l1_w; j++) {
            l1.weights[i][j] <== w1[i][j];
        }
        l1.biases[i] <== b1[i];
    }
    // l1.weights <== w1;
    // l1.biases <== b1;
    for (var k = 0; k < l0_h; k++) {
        l1.in[k] <== l0.out[k];
    }
    // l1.in <== l0.out;

    component l2 = fc(l2_w, l2_h);
    signal input w2[l2_h][l2_w];
    signal input b2[l2_h];
    for (var i = 0; i < l2_h; i++) {
        for (var j = 0; j < l2_w; j++) {
            l2.weights[i][j] <== w2[i][j];
        }
        l2.biases[i] <== b2[i];
    }
    // l2.weights <== w2;
    // l2.biases <== b2;
    for (var k = 0; k < l1_h; k++) {
        l2.in[k] <== l1.out[k];
    }
    // l2.in <== l1.out;

    component l3 = fc_no_relu(l3_w, l3_h);
    signal input w3[l3_h][l3_w];
    signal input b3[l3_h];
    for (var i = 0; i < l3_h; i++) {
        for (var j = 0; j < l3_w; j++) {
            l3.weights[i][j] <== w3[i][j];
        }
        l3.biases[i] <== b3[i];
    }
    // l2.weights <== w2;
    // l2.biases <== b2;
    for (var k = 0; k < l2_h; k++) {
        l3.in[k] <== l2.out[k];
    }
    // l2.in <== l1.out;

    for (var k = 0; k < out_len; k++) {
        out[k] <== l3.out[k];
    }
    // out <== l2.out;
}

component main = network();