import { WebPlugin } from '@capacitor/core';
import type { IzelsUsefulPluginPlugin } from './definitions';

export class IzelsUsefulPluginWeb extends WebPlugin implements IzelsUsefulPluginPlugin {
  async echoMe(): Promise<{ value: string }> {
    console.log('Hello World from web version!');
    return { value: 'izel' };
  }

  async showToast(options: { message: string }): Promise<void> {
    alert(`Web Toast: ${options.message}`);
  }
}

// NOTE: Initial generated one:
//
// import { WebPlugin } from '@capacitor/core';
//
// import type { IzelsUsefulPluginPlugin } from './definitions';
//
// export class IzelsUsefulPluginWeb extends WebPlugin implements IzelsUsefulPluginPlugin {
//   async echo(options: { value: string }): Promise<{ value: string }> {
//     console.log('ECHO', options);
//     return options;
//   }
// }
