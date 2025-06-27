# capacitor-izels-example-plugin

Basic cross-platform plugin with echo and native UI call

## Install

```bash
npm install capacitor-izels-example-plugin
npx cap sync
```

## To build/run this project/example-app locally:

```bash
npm install
npm run build

cd example-app

nix-shell # NOTE: essential to set ENV variables, system dependencies from shell.nix
npx cap sync
npx cap run android # Make sure you're already running a Virtual Device on Android Studio or have a connected device with adb
```


## API

<docgen-index>

* [`echoMe()`](#echome)
* [`showToast(...)`](#showtoast)

</docgen-index>

<docgen-api>
<!--Update the source file JSDoc comments and rerun docgen to update the docs below-->

### echoMe()

```typescript
echoMe() => Promise<{ value: string; }>
```

**Returns:** <code>Promise&lt;{ value: string; }&gt;</code>

--------------------


### showToast(...)

```typescript
showToast(options: { message: string; }) => Promise<void>
```

| Param         | Type                              |
| ------------- | --------------------------------- |
| **`options`** | <code>{ message: string; }</code> |

--------------------

</docgen-api>
