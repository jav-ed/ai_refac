package com.example.legacy;

import com.example.util.Helper;

public class JavaCaller {
    public String run() {
        Helper helper = new Helper("j:");
        return helper.decorate("x") + helper.getCounter();
    }
}
