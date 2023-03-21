import {Component, OnInit} from '@angular/core';
import {WindowService} from './window.service';
import {AudioCacheIndex} from './audio-data';

@Component({
  selector: 'app-window',
  templateUrl: './window.component.html',
  styleUrls: ['./window.component.less']
})
export class WindowComponent implements OnInit {
  inputMessage = '';
  audios: AudioCacheIndex[] = [];

  constructor(private service: WindowService) {
  }

  ngOnInit(): void {
    this.service.listAudios().subscribe(data => {
      data.sort((a, b) => b.time.localeCompare(a.time));
      this.audios.push(...data);
    });
  }

  play() {
    this.service.generateAudio(this.inputMessage)
      .subscribe((res) => {
        if (!!res) {
          this.audios.unshift(res);
        }
      });
  }
}
