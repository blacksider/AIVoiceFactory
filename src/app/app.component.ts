import {Component, OnDestroy, OnInit} from "@angular/core";
import {listen} from "@tauri-apps/api/event";
import {WindowService} from "./window/window.service";

@Component({
  selector: "app-root",
  templateUrl: "./app.component.html",
  styleUrls: ["./app.component.less"],
})
export class AppComponent implements OnInit, OnDestroy {
  private unListenRegText?: () => void;

  constructor(private windowService: WindowService) {
  }

  ngOnInit(): void {
    listen('on_audio_recognize_text', (event) => {
      const text = event.payload as string;
      this.windowService.handleRegText(text);
    })
      .then((fn) => {
        this.unListenRegText = fn;
      });
  }

  ngOnDestroy(): void {
    if (this.unListenRegText) {
      this.unListenRegText();
    }
  }
}
