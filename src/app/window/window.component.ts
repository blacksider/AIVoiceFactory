import { Component } from '@angular/core';
import {invoke} from '@tauri-apps/api/tauri';

@Component({
  selector: 'app-window',
  templateUrl: './window.component.html',
  styleUrls: ['./window.component.less']
})
export class WindowComponent {
  greetingMessage = "";

  inputMessage = "";

  greet(event: SubmitEvent, name: string): void {
    event.preventDefault();

    // Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
    invoke<string>("greet", {name}).then((text) => {
      this.greetingMessage = text;
    });
  }
}
