import {Component, NgZone, OnDestroy, OnInit} from '@angular/core';
import {WindowService} from './window.service';
import {AudioCacheDetail, AudioCacheIndex} from './audio-data';
import {NzResizeEvent} from 'ng-zorro-antd/resizable';
import {VoiceRecognitionService} from "../voice-recognition/voice-recognition.service";
import {listen} from "@tauri-apps/api/event";
import {WebviewWindow} from "@tauri-apps/api/window";

@Component({
  selector: 'app-window',
  templateUrl: './window.component.html',
  styleUrls: ['./window.component.less']
})
export class WindowComponent implements OnInit, OnDestroy {
  siderWidth = 100;
  inputMessage = '';
  audios: AudioCacheIndex[] = [];
  audioDetails: { [key: string]: AudioCacheDetail };
  resizeFrame = -1;
  syncOnTextRecognize = false;

  isRecording = false;

  private unListenRegText?: () => void;
  private unListenRecorderState?: () => void;
  private recordingWindow?: WebviewWindow;

  constructor(private service: WindowService,
              private voiceRecognitionService: VoiceRecognitionService,
              private ngZone: NgZone) {
    this.audioDetails = {};
    const windowWidth = window.innerWidth;
    if (windowWidth >= 500) {
      this.siderWidth = 200;
    } else if (windowWidth >= 900) {
      this.siderWidth = 400;
    } else if (windowWidth < 200) {
      this.siderWidth = windowWidth / 2;
    }
  }

  ngOnInit(): void {
    this.service.listAudios().subscribe(data => {
      data.sort((a, b) => b.time.localeCompare(a.time));
      this.audios.push(...data);
    });
    listen('on_audio_recognize_text', (event) => {
      const text = event.payload as string;
      this.updateVoiceRegText(text);
    })
      .then((fn) => {
        this.unListenRegText = fn;
      });
    this.voiceRecognitionService.isRecorderRecording().subscribe(ret => {
      this.isRecording = ret;
      this.checkRecordingState();
    });
    listen('on_recorder_state_change', (event) => {
      this.isRecording = event.payload as boolean;
      this.checkRecordingState();
    })
      .then((fn) => {
        this.unListenRecorderState = fn;
      });
  }

  ngOnDestroy(): void {
    if (this.unListenRegText) {
      this.unListenRegText();
    }
    if (this.unListenRecorderState) {
      this.unListenRecorderState();
    }
    /*if (this.recordingWindow) {
      this.recordingWindow.close();
    }*/
  }

  private checkRecordingState() {
    /*if (this.isRecording) {
      if (this.recordingWindow) {
        this.recordingWindow.show();
        return;
      }
      const webview = new WebviewWindow('recordingWindow', {
        url: '/recording'
      });
      webview.once('tauri://created', function () {
        // webview window successfully created
        console.log('recording-window-opened');
      });
      webview.once('tauri://error', function (e) {
        // an error happened creating the webview window
        console.log('recording-window-error', e);
      });
      this.recordingWindow = webview;
    } else {
      if (this.recordingWindow) {
        this.recordingWindow.hide();
      }
    }*/
  }

  private updateVoiceRegText(text: string) {
    if (!text) {
      return;
    }
    this.ngZone.run(() => {
      this.inputMessage = text;
      if (this.syncOnTextRecognize) {
        this.generate();
      }
    });
  }

  generate() {
    this.service.generateAudio(this.inputMessage)
      .subscribe((res) => {
        if (!!res) {
          this.audios.unshift(res);
        }
      });
  }

  onExpandItem(active: boolean, item: AudioCacheIndex) {
    if (active && !this.audioDetails.hasOwnProperty(item.name)) {
      this.service.getAudioDetail(item.name)
        .subscribe(value => {
          this.audioDetails[item.name] = value;
        });
    }
  }

  onSideResize({width}: NzResizeEvent) {
    cancelAnimationFrame(this.resizeFrame);
    this.resizeFrame = requestAnimationFrame(() => {
      this.siderWidth = width!;
    });
  }

  playAudio(index: string) {
    this.service.playAudio(index).subscribe(() => {
    });
  }
}
