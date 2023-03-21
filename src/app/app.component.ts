import {Component, OnInit} from "@angular/core";
import {WindowService} from './window/window.service';
import {timer} from 'rxjs';

@Component({
  selector: "app-root",
  templateUrl: "./app.component.html",
  styleUrls: ["./app.component.less"],
})
export class AppComponent implements OnInit {
  constructor(private windowService: WindowService) {
  }

  ngOnInit(): void {
    timer(0, 60000)
      .subscribe(_ => {
        console.log("check audio caches");
        this.windowService.checkAudioCaches();
      })
  }
}
