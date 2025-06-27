export interface IzelsUsefulPluginPlugin {
  echoMe(): Promise<{ value: string }>;
  showToast(options: { message: string }): Promise<void>;
}
