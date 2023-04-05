import { ComponentFixture, TestBed } from '@angular/core/testing';

import { RecordingPopupComponent } from './recording-popup.component';

describe('RecordingPopupComponent', () => {
  let component: RecordingPopupComponent;
  let fixture: ComponentFixture<RecordingPopupComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [ RecordingPopupComponent ]
    })
    .compileComponents();

    fixture = TestBed.createComponent(RecordingPopupComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
