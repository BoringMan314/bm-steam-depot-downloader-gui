import {readText} from "@tauri-apps/plugin-clipboard-manager";
import {useContext, useState} from "preact/hooks";
import {AppContext} from "../context";
import {Icon} from "@iconify-icon/react";

export function ClipBoardParserButton({disabled}: {disabled: boolean}) {
	const context = useContext(AppContext);
	const [modal, setModal] = useState(false);
	
	const appRe = /-app\s+(\d+)/i;
	const depotRe = /-depot\s+(\d+)/i;
	const manifestRe = /-manifest\s+(\d+)/i;	
	
	const process = async () => {
		const content: string = await readText();
		
		const appMatch = appRe.exec(content);
		const depotMatch = depotRe.exec(content);
		const manifestMatch = manifestRe.exec(content);
		
		if (appMatch && depotMatch && manifestMatch) {
			context.appId![1](appMatch[1]);
			context.depotId![1](depotMatch[1]);
			context.manifestId![1](manifestMatch[1]);
		} else {
			alert("剪貼簿的內容格式不正確。\n正確格式範例：-app 346900 -depot 346903 -manifest 1203243898820547407");
		}
	};
	
	return (
		<div class={`${(disabled && !modal) && "active:scale-102"}`}>
			<div hidden={!modal} onClick={(e: MouseEvent) => { e.stopPropagation(); setModal(false); }} class="flex fixed top-0 left-0 z-10 items-center w-1/2 h-full">
				<div class="flex flex-col py-1 px-2 mx-auto max-w-sm rounded-md -translate-y-8 bg-blue-900 border-gray-300 border">
					<p class="mb-2 text-2xl font-semibold underline underline-offset-4">自動填入的運作方式</p>
					<p>在 SteamDB 的 manifest 總覽頁面中，可以設定<i>複製格式</i>。</p>
					<img src="../../assets/steamdb_copy.png" />
					<p>
						只要把格式設為 DepotDownloader，本程式就能解析剪貼簿的內容，
						並自動填入下方的表單欄位。
					</p>
				</div>
			</div>
			<div class="flex">
				{/* https://stackoverflow.com/a/73923373 */}
				<button disabled={disabled} onClick={() => void process()} type="button" class="inline-flex items-center py-0.5 px-1 font-medium text-white rounded-l-md border border-r-0 border-gray-600 transition-transform disabled:cursor-not-allowed rounded-e-0 bg-blue-900/90 text-md hover:bg-blue-900/65">
					<Icon icon="mdi:clipboard-text-search" width="20" height="20" />自動填入
				</button>
				<button onClick={() => setModal(true)} type="button" class="inline-flex items-center py-1 px-2 font-medium text-white rounded-r-md border border-r-0 border-gray-600 transition-transform disabled:cursor-not-allowed rounded-e-0 bg-blue-900/90 text-md hover:bg-blue-900/65">
					<Icon icon="material-symbols:help" width="20" height="20" />
				</button>
			</div>
			
		</div>
	);
}