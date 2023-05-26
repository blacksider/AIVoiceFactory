import {Component, ElementRef, NgZone, OnDestroy, OnInit, ViewChild} from '@angular/core';
import {listen} from "@tauri-apps/api/event";

@Component({
  selector: 'app-recording-popup',
  templateUrl: './recording-popup.component.html',
  styleUrls: ['./recording-popup.component.less']
})
export class RecordingPopupComponent implements OnInit, OnDestroy {
  @ViewChild('recorder')
  private recorderContainer: ElementRef<HTMLDivElement> | undefined;

  recordingState: string = 'Recording';
  pendingMessage: string = '等待输入...';
  messages: string[] = [];
  private unListenState?: () => void;
  private unListenMsg?: () => void;

  constructor(private ngZone: NgZone) {
  }

  ngOnInit(): void {
    listen('on_recoding_state', (event) => {
      const state = event.payload as string;
      this.ngZone.run(() => {
        this.recordingState = state;
        // detect speech
        if (this.recordingState === "DetectSpeech") {
          this.pendingMessage = '检测到语音，正在识别...';
        } else {
          this.pendingMessage = '等待输入...';
        }
      });
    })
      .then((fn) => {
        this.unListenState = fn;
      });
    listen('on_recoding_recognize_text', (event) => {
      const text = event.payload as string;
      this.ngZone.run(() => {
        this.messages.push(`用户：${text}`);
        if (this.messages.length > 100) {
          // remove messages before 100 messages
          this.messages = this.messages.slice(0, this.messages.length - 100);
        }
        setTimeout(() => {
          this.scrollToBottom();
        });
      });
    })
      .then((fn) => {
        this.unListenMsg = fn;
      });
  }

  ngOnDestroy(): void {
    if (this.unListenState) {
      this.unListenState();
    }
    if (this.unListenMsg) {
      this.unListenMsg();
    }
  }

  private scrollToBottom(): void {
    if (!this.recorderContainer) {
      return;
    }
    this.recorderContainer.nativeElement.scrollTop = this.recorderContainer.nativeElement.scrollHeight;
  }
}
