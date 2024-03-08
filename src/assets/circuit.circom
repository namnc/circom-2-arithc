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
    sign <== in < 0;
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
    var l0_h = 32; 

    var l1_w = l0_h;
    var l1_h = 64;

    var l2_w = l1_h;
    var l2_h = 128;

    var l3_w = l2_h; 
    var l3_h = 256;

    var l4_w = l3_h; 
    var l4_h = 512;

    var l5_w = l4_h; 
    var l5_h = 1024;

    var l6_w = l5_h; 
    var l6_h = 2048;

    var l7_w = l6_h; 
    var l7_h = out_len;

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

    component l3 = fc(l3_w, l3_h);
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

    component l4 = fc(l4_w, l4_h);
    signal input w4[l4_h][l4_w];
    signal input b4[l4_h];
    for (var i = 0; i < l4_h; i++) {
        for (var j = 0; j < l4_w; j++) {
            l4.weights[i][j] <== w4[i][j];
        }
        l4.biases[i] <== b4[i];
    }
    for (var k = 0; k < l4_h; k++) {
        l4.in[k] <== l3.out[k];
    }

    component l5 = fc(l5_w, l5_h);
    signal input w5[l5_h][l5_w];
    signal input b5[l5_h];
    for (var i = 0; i < l5_h; i++) {
        for (var j = 0; j < l5_w; j++) {
            l5.weights[i][j] <== w5[i][j];
        }
        l5.biases[i] <== b5[i];
    }
    for (var k = 0; k < l5_h; k++) {
        l5.in[k] <== l4.out[k];
    }

    component l6 = fc(l6_w, l6_h);
    signal input w6[l6_h][l6_w];
    signal input b6[l6_h];
    for (var i = 0; i < l6_h; i++) {
        for (var j = 0; j < l6_w; j++) {
            l6.weights[i][j] <== w6[i][j];
        }
        l6.biases[i] <== b6[i];
    }
    for (var k = 0; k < l6_h; k++) {
        l6.in[k] <== l5.out[k];
    }

    component l7 = fc(l7_w, l7_h);
    signal input w7[l7_h][l7_w];
    signal input b7[l7_h];
    for (var i = 0; i < l7_h; i++) {
        for (var j = 0; j < l7_w; j++) {
            l7.weights[i][j] <== w7[i][j];
        }
        l7.biases[i] <== b7[i];
    }
    for (var k = 0; k < l7_h; k++) {
        l7.in[k] <== l6.out[k];
    }

    // component l8 = fc_no_relu(l8_w, l8_h);
    // signal input w8[l8_h][l8_w];
    // signal input b8[l8_h];
    // for (var i = 0; i < l8_h; i++) {
    //     for (var j = 0; j < l8_w; j++) {
    //         l8.weights[i][j] <== w8[i][j];
    //     }
    //     l8.biases[i] <== b8[i];
    // }
    // // l3.weights <== w2;
    // // l3.biases <== b2;
    // for (var k = 0; k < l8_h; k++) {
    //     l8.in[k] <== l7.out[k];
    // }
    // // l3.in <== l1.out;

    for (var k = 0; k < out_len; k++) {
        out[k] <== l7.out[k];
    }
    // out <== l2.out;
}

component main = network();