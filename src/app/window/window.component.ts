import {Component, OnInit} from '@angular/core';
import {WindowService} from './window.service';
import {AudioCacheDetail, AudioCacheIndex} from './audio-data';
import {NzResizeEvent} from 'ng-zorro-antd/resizable';

@Component({
  selector: 'app-window',
  templateUrl: './window.component.html',
  styleUrls: ['./window.component.less']
})
export class WindowComponent implements OnInit {
  siderWidth = 100;
  inputMessage = '';
  audios: AudioCacheIndex[] = [];
  audioDetails: { [key: string]: AudioCacheDetail };
  resizeFrame = -1;

  constructor(private service: WindowService) {
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
