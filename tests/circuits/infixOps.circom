pragma circom 2.1.0;

template infixOps() {
    signal input x0;
    signal input x1;
    signal input x2;
    signal input x3;
    signal input x4;
    signal input x5;

    signal output mul_2_3;
    // signal output div_4_3; // unsupported for NumberU32
    signal output idiv_4_3;
    signal output add_3_4;
    signal output sub_4_1;
    signal output pow_2_4;
    signal output mod_5_3;
    signal output shl_5_1;
    signal output shr_5_1;
    signal output leq_2_3;
    signal output leq_3_3;
    signal output leq_4_3;
    signal output geq_2_3;
    signal output geq_3_3;
    signal output geq_4_3;
    signal output lt_2_3;
    signal output lt_3_3;
    signal output lt_4_3;
    signal output gt_2_3;
    signal output gt_3_3;
    signal output gt_4_3;
    signal output eq_2_3;
    signal output eq_3_3;
    signal output neq_2_3;
    signal output neq_3_3;
    signal output or_0_1;
    signal output and_0_1;
    signal output bit_or_1_3;
    signal output bit_and_1_3;
    signal output bit_xor_1_3;

    mul_2_3 <== x2 * x3;
    // div_4_3 <== x4 / x3;
    idiv_4_3 <== x4 \ x3;
    add_3_4 <== x3 + x4;
    sub_4_1 <== x4 - x1;
    pow_2_4 <== x2 ** x4;
    mod_5_3 <== x5 % x3;
    shl_5_1 <== x5 << x1;
    shr_5_1 <== x5 >> x1;
    leq_2_3 <== x2 <= x3;
    leq_3_3 <== x3 <= x3;
    leq_4_3 <== x4 <= x3;
    geq_2_3 <== x2 >= x3;
    geq_3_3 <== x3 >= x3;
    geq_4_3 <== x4 >= x3;
    lt_2_3 <== x2 < x3;
    lt_3_3 <== x3 < x3;
    lt_4_3 <== x4 < x3;
    gt_2_3 <== x2 > x3;
    gt_3_3 <== x3 > x3;
    gt_4_3 <== x4 > x3;
    eq_2_3 <== x2 == x3;
    eq_3_3 <== x3 == x3;
    neq_2_3 <== x2 != x3;
    neq_3_3 <== x3 != x3;
    or_0_1 <== x0 || x1;
    and_0_1 <== x0 && x1;
    bit_or_1_3 <== x1 | x3;
    bit_and_1_3 <== x1 & x3;
    bit_xor_1_3 <== x1 ^ x3;
}

component main = infixOps();
