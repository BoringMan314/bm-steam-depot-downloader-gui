import { Icon } from "@iconify-icon/react";
import { useContext } from "preact/hooks";
import { AppContext, AppSettings } from "../context";
import { BooleanInput, TextInput } from "./FormInput";
import { ComponentChildren } from "preact";

export function Settings() {
	const context = useContext(AppContext);
	const [settings, setSettings] = context.appSettings!;
	
	const setOutputDirectoryName = (newValue: AppSettings["outputDirectoryName"]) => setSettings(oldSettings => { return ({ ...oldSettings, outputDirectoryName: newValue }); });
	const setOutputDirectoryMode = (newValue: AppSettings["outputDirectoryMode"]) => setSettings(oldSettings => { return ({ ...oldSettings, outputDirectoryMode: newValue }); });
	const setNoMobileAuth = (newValue: AppSettings["noMobileAuth"]) => setSettings(oldSettings => { return ({ ...oldSettings, noMobileAuth: newValue }); });
	
	
	return (
		<>
			<div class="flex justify-center w-full">
				<button onClick={() => context.showSettings![1](s => !s)} type="button" class="inline-flex absolute left-0 gap-1 items-center py-1 px-2 text-xl font-semibold bg-green-600 rounded translate-x-1 translate-y-1 hover:bg-green-700 active:bg-green-800">
					<Icon icon="icon-park-solid:back" width="20" height="20"/>返回
				</button>
				<div class="mb-1 text-4xl font-bold text-center font-['Hubot_Sans']">
					設定
				</div>
			</div>
			<div class="px-[10%] flex-col flex gap-3">
				<SettingsGroup title="資料夾名稱" description={"DepotDownloader 下載遊戲時所建立的資料夾名稱。\n這個資料夾會建立在你選擇的輸出資料夾底下。"}>
					<div class="flex gap-px mb-2 rounded-md shadow-xs">
						<button
							onClick={() => { setOutputDirectoryMode("Manifest ID"); }}
							type="button"
							class={`border-black px-3 py-2  border border-r-0 font-medium text-md disabled:bg-red-500/70 disabled:pointer-events-none inline-flex items-center justify-center rounded-s-lg focus:z-10 focus:ring-2 focus:ring-blue-500 ${settings?.outputDirectoryMode === "Manifest ID" ? "bg-blue-500 underline border-0 outline-gray-300 outline shadow-[0px_0px_29px_-8px_cornflowerblue] hover:bg-blue-600 active:bg-blue-700" : "hover:bg-blue-800 active:bg-blue-900 bg-blue-700"}`}>
							用 Manifest ID
					  </button>
						<button
							onClick={() => setOutputDirectoryMode("Custom")}
							type="button"
							class={`border-black px-3 py-2  border border-l-0 font-medium text-md disabled:bg-red-500/70 disabled:pointer-events-none inline-flex items-center justify-center rounded-e-lg focus:z-10 focus:ring-2 focus:ring-blue-500 ${settings?.outputDirectoryMode === "Custom" ? "bg-blue-500 underline border-0 outline-gray-300 outline shadow-[0px_0px_29px_-8px_cornflowerblue] hover:bg-blue-600 active:bg-blue-700" : "hover:bg-blue-800 active:bg-blue-900 bg-blue-700"}`}>
							自訂名稱
					  </button>
					</div>
					<TextInput className="max-w-1/2" disabled={settings.outputDirectoryMode === "Manifest ID"} id="directoryName" placeholder="DepotDownloader 輸出資料夾名稱" initialValue={settings.outputDirectoryName} onInput={setOutputDirectoryName} />
				</SettingsGroup>
				<SettingsGroup title="不使用手機驗證" description={"改為手動輸入兩步驟驗證碼，而不是在 Steam 手機 App 上點選確認。"}>
					<BooleanInput id="qrCode" initialValue={settings.noMobileAuth} onInput={setNoMobileAuth} />
				</SettingsGroup>
			</div>
		</>
	);
}

function SettingsGroup({ title, description, children }: { title: string, description: string, children: ComponentChildren }) {
	return (
		<div>
			<div class="text-2xl font-semibold text-white">
				{title}
			</div>
			{/* https://stackoverflow.com/a/76313520 */}
			<p class="mb-3 text-white whitespace-pre-wrap">
				{description}
			</p>
			{children}
		</div>
	);
}