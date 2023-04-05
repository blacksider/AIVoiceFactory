import { ComponentFixture, TestBed } from '@angular/core/testing';

import { KeyRecorderComponent } from './key-recorder.component';

describe('KeyRecorderComponent', () => {
  let component: KeyRecorderComponent;
  let fixture: ComponentFixture<KeyRecorderComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ KeyRecorderComponent ]
    })
    .compileComponents();

    fixture = TestBed.createComponent(KeyRecorderComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
