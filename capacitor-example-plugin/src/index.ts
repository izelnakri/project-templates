import { registerPlugin } from '@capacitor/core';

import type { IzelsUsefulPluginPlugin } from './definitions';

const IzelsUsefulPlugin = registerPlugin<IzelsUsefulPluginPlugin>('IzelsUsefulPlugin', {
  web: () => import('./web').then((m) => new m.IzelsUsefulPluginWeb()),
});

export * from './definitions';
export { IzelsUsefulPlugin };
