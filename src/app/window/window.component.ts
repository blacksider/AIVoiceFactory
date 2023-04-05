import {Component, NgZone, OnDestroy, OnInit} from '@angular/core';
import {WindowService} from './window.service';
import {AudioCacheDetail, AudioCacheIndex} from './audio-data';
import {NzResizeEvent} from 'ng-zorro-antd/resizable';
import {VoiceRecognitionService} from "../voice-recognition/voice-recognition.service";
import {listen} from "@tauri-apps/api/event";
import {LocalStorageService} from "../local-storage.service";

const KEY_SYNC_STATE = "syncOnTextRecognize";
const KEY_WIDTH = "windowWidth";

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

  constructor(private service: WindowService,
              private voiceRecognitionService: VoiceRecognitionService,
              private localStorage: LocalStorageService,
              private ngZone: NgZone) {
    this.audioDetails = {};
    const storedWidth = this.localStorage.get(KEY_WIDTH);
    if (!!storedWidth) {
      this.siderWidth = Number.parseInt(storedWidth, 10);
    } else {
      let windowWidth = window.innerWidth;
      this.siderWidth = windowWidth / 2;
    }
    const storedSyncState = this.localStorage.get(KEY_SYNC_STATE);
    if (!!storedSyncState) {
      this.syncOnTextRecognize = new Boolean(storedSyncState).valueOf();
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
    });
    listen('on_recorder_state_change', (event) => {
      this.isRecording = event.payload as boolean;
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
      this.localStorage.set(KEY_WIDTH, this.siderWidth + "");
    });
  }

  playAudio(index: string) {
    this.service.playAudio(index).subscribe(() => {
    });
  }

  onSyncValueChange() {
    if (this.syncOnTextRecognize) {
      this.localStorage.set(KEY_SYNC_STATE, "true");
    } else {
      this.localStorage.set(KEY_SYNC_STATE, "");
    }
  }
}
