import {Component, HostListener, NgZone, OnDestroy, OnInit} from '@angular/core';
import {WindowService} from './window.service';
import {AudioCacheDetail, AudioCacheIndex, AudioRegEvent} from './audio-data';
import {NzResizeEvent} from 'ng-zorro-antd/resizable';
import {VoiceRecognitionService} from "../voice-recognition/voice-recognition.service";
import {listen} from "@tauri-apps/api/event";
import {LocalStorageService} from "../local-storage.service";
import {debounceTime, Subject, takeUntil} from "rxjs";
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
  isRecording = false;

  private unListenRecorderState?: () => void;
  private unListenAudioGen?: () => void;
  private windowResize = new Subject<number>();
  private unSub = new Subject();
  private confirmModal?: NzModalRef;
  private currentWindowSize = window.innerWidth;

  constructor(private service: WindowService,
              private voiceRecognitionService: VoiceRecognitionService,
              private localStorage: LocalStorageService,
              private modal: NzModalService,
              private ngZone: NgZone) {
    this.audioDetails = {};
  }

  @HostListener('window:resize', ['$event'])
  onResize() {
    let windowWidth = window.innerWidth;
    this.windowResize.next(windowWidth);
  }

  ngOnInit(): void {
    this.windowResize
      .asObservable()
      .pipe(
        takeUntil(this.unSub),
        debounceTime(200)
      )
      .subscribe(windowWidth => {
        let ratio = this.siderWidth / this.currentWindowSize;
        this.siderWidth = windowWidth * ratio;
        this.currentWindowSize = windowWidth;
      });
    const storedWidth = this.localStorage.get(KEY_WIDTH);
    if (!!storedWidth) {
      this.siderWidth = Number.parseInt(storedWidth, 10);
      if (this.siderWidth >= this.currentWindowSize) {
        this.siderWidth = this.currentWindowSize / 2;
      }
    } else {
      this.siderWidth = this.currentWindowSize / 2;
    }
    this.loadAudios();
    this.service.listenRegText()
      .pipe(takeUntil(this.unSub))
      .subscribe(value => {
        this.updateVoiceRegText(value);
      });
    listen('on_audio_generated', (event) => {
      const audio = event.payload as AudioCacheIndex;
      this.ngZone.run(() => {
        this.audios.unshift(audio);
      });
    }).then(fn => {
      this.unListenAudioGen = fn;
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
    this.windowResize.complete();
    if (this.unListenRecorderState) {
      this.unListenRecorderState();
    }
    if (this.unListenAudioGen) {
      this.unListenAudioGen();
    }
    if (this.confirmModal) {
      this.confirmModal.destroy();
    }
  }

  private updateVoiceRegText(event: AudioRegEvent) {
    if (!event.text) {
      return;
    }
    this.ngZone.run(() => {
      this.inputMessage = event.text;
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
