pragma circom 2.0.0;

template MedicalAccess() {
    signal input recordHash;
    signal input userHash;
    signal input authorityHash;

    signal output valid;

    valid <== (recordHash * userHash * authorityHash) != 0;
}

component main = MedicalAccess();
