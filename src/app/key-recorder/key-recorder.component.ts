import {Component, OnDestroy} from '@angular/core';
import {ControlValueAccessor, NG_VALUE_ACCESSOR} from "@angular/forms";

@Component({
  selector: 'app-key-recorder',
  templateUrl: './key-recorder.component.html',
  styleUrls: ['./key-recorder.component.less'],
  providers: [
    {
      provide: NG_VALUE_ACCESSOR,
      multi: true,
      useExisting: KeyRecorderComponent
    }
  ]
})
export class KeyRecorderComponent implements ControlValueAccessor, OnDestroy {
  disabled = false;
  keys = "";

  editing = false;

  metaKeys = ['CONTROL', 'SHIFT', 'ALT'];

  private onWatchKeys = this.watchKeys.bind(this);

  ngOnDestroy(): void {
    this.stopWatchKeys();
  }

  onChange = (keys: string) => {
  };

  onTouched = () => {
  };

  registerOnChange(fn: any): void {
    this.onChange = fn;
  }

  registerOnTouched(fn: any): void {
    this.onTouched = fn;
  }

  setDisabledState(isDisabled: boolean): void {
    this.disabled = isDisabled;
  }

  writeValue(keys: string): void {
    this.keys = keys;
  }

  toEdit() {
    if (this.editing) {
      this.editing = false;
      this.stopWatchKeys();
    } else {
      this.editing = true;
      this.startWatchKeys();
    }
  }

  private watchKeys(event: KeyboardEvent) {
    let keydown = [];
    if (event.ctrlKey) {
      keydown.push('CTRL');
    }
    if (event.shiftKey) {
      keydown.push('SHIFT');
    }
    if (event.altKey) {
      keydown.push('ALT');
    }
    if (event.key && this.metaKeys.indexOf(event.key.toUpperCase()) === -1) {
      keydown.push(event.key.toUpperCase());
    }
    this.keys = keydown.join('+');
    this.onTouched();
    this.onChange(this.keys);
  }

  private startWatchKeys() {
    window.addEventListener('keydown', this.onWatchKeys, false);
  }

  private stopWatchKeys() {
    window.removeEventListener('keydown', this.onWatchKeys, false);
  }
}
