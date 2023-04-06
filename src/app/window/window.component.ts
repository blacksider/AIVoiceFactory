import {Component, NgZone, OnDestroy, OnInit} from '@angular/core';
import {WindowService} from './window.service';
import {AudioCacheDetail, AudioCacheIndex} from './audio-data';
import {NzResizeEvent} from 'ng-zorro-antd/resizable';
import {VoiceRecognitionService} from "../voice-recognition/voice-recognition.service";
import {listen} from "@tauri-apps/api/event";
import {LocalStorageService} from "../local-storage.service";
import {Subject, takeUntil} from "rxjs";
import {NzModalRef, NzModalService} from "ng-zorro-antd/modal";

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

  private unListenRecorderState?: () => void;
  private unSub = new Subject();
  private confirmModal?: NzModalRef;

  constructor(private service: WindowService,
              private voiceRecognitionService: VoiceRecognitionService,
              private localStorage: LocalStorageService,
              private modal: NzModalService,
              private ngZone: NgZone) {
    this.audioDetails = {};
  }

  ngOnInit(): void {
    const storedWidth = this.localStorage.get(KEY_WIDTH);
    if (!!storedWidth) {
      this.siderWidth = Number.parseInt(storedWidth, 10);
    } else {
      let windowWidth = window.innerWidth;
      this.siderWidth = windowWidth / 2;
    }
    this.syncOnTextRecognize = this.service.getSyncOnTextState();
    this.loadAudios();
    this.service.listenRegText()
      .pipe(takeUntil(this.unSub))
      .subscribe(value => {
        this.updateVoiceRegText(value);
      });
    this.service.listenAudioIndex()
      .pipe(takeUntil(this.unSub))
      .subscribe(value => {
        this.ngZone.run(() => {
          this.audios.unshift(value);
        });
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
    this.unSub.next({});
    this.unSub.complete();
    if (this.unListenRecorderState) {
      this.unListenRecorderState();
    }
    if (this.confirmModal) {
      this.confirmModal.destroy();
    }
  }

  private updateVoiceRegText(text: string) {
    if (!text) {
      return;
    }
    this.ngZone.run(() => {
      this.inputMessage = text;
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
    item.active = active;
    if (active && !this.audioDetails.hasOwnProperty(item.name)) {
      this.loadAudioDetail(item);
    }
  }

  private loadAudioDetail(item: AudioCacheIndex) {
    this.service.getAudioDetail(item.name)
      .subscribe(value => {
        this.audioDetails[item.name] = value;
      });
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
    this.service.updateSyncOnTextState(this.syncOnTextRecognize);
  }

  removeAudio(item: AudioCacheIndex) {
    this.confirmModal = this.modal.confirm({
      nzTitle: '警告',
      nzContent: '确认删除此记录？',
      nzOnOk: () => this.service.deleteAudio(item.name).then(() => {
        this.loadAudios();
      })
    });
  }

  private loadAudios() {
    const original = new Map(this.audios.map(obj => [obj.name, obj]));
    this.service.listAudios().subscribe(data => {
      const audios = [];
      data.sort((a, b) => b.time.localeCompare(a.time));
      audios.push(...data);
      for (let audio of audios) {
        if (original.has(audio.name)) {
          audio.active = original.get(audio.name)!.active;
        }
      }
      this.audios = audios;
    });
  }

  collapseAudios(collapse: boolean) {
    for (let audio of this.audios) {
      if (!collapse && !audio.active) {
        this.loadAudioDetail(audio);
      }
      audio.active = !collapse;
    }
  }
}
