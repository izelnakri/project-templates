package com.izel.usefulplugin;

import android.widget.Toast;
import android.util.Log;

import com.getcapacitor.Plugin;
import com.getcapacitor.PluginCall;
import com.getcapacitor.PluginMethod;
import com.getcapacitor.annotation.CapacitorPlugin;

@CapacitorPlugin(name = "IzelsUsefulPlugin")
public class IzelsUsefulPluginPlugin extends Plugin {

    @PluginMethod
    public void echoMe(PluginCall call) {
        Log.i("IzelsUsefulPlugin", "Hello World");
        call.resolve(new com.getcapacitor.JSObject().put("value", "izot"));
    }

    @PluginMethod
    public void showToast(PluginCall call) {
        String message = call.getString("message");
        Toast.makeText(getContext(), message != null ? message : "No message", Toast.LENGTH_SHORT).show();
        call.resolve();
    }
}

// NOTE: Initially generated file:
//
// package com.izel.usefulplugin;
//
// import com.getcapacitor.JSObject;
// import com.getcapacitor.Plugin;
// import com.getcapacitor.PluginCall;
// import com.getcapacitor.PluginMethod;
// import com.getcapacitor.annotation.CapacitorPlugin;
//
// @CapacitorPlugin(name = "IzelsUsefulPlugin")
// public class IzelsUsefulPluginPlugin extends Plugin {
//
//     private IzelsUsefulPlugin implementation = new IzelsUsefulPlugin();
//
//     @PluginMethod
//     public void echo(PluginCall call) {
//         String value = call.getString("value");
//
//         JSObject ret = new JSObject();
//         ret.put("value", implementation.echo(value));
//         call.resolve(ret);
//     }
// }
