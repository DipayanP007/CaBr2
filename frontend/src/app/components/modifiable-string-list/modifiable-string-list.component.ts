import { CdkDragDrop, moveItemInArray } from '@angular/cdk/drag-drop';
import { Component, EventEmitter, Input, OnInit, Output } from '@angular/core';
import { FormArray, FormBuilder, FormGroup } from '@angular/forms';
import { Observable } from 'rxjs';

@Component({
  selector: 'app-modifiable-string-list',
  templateUrl: './modifiable-string-list.component.html',
  styleUrls: ['./modifiable-string-list.component.scss'],
})
export class ModifiableStringListComponent implements OnInit {
  @Output()
  elementEmitter = new EventEmitter<string[]>();

  @Input()
  title = '';

  @Input()
  elements!: Observable<string[]>;

  form: FormGroup;

  addHover = false;

  constructor(private formBuilder: FormBuilder) {
    this.form = this.formBuilder.group({});
  }

  get controlElements(): FormArray {
    return this.form.get('elements') as FormArray;
  }

  ngOnInit(): void {
    this.elements.subscribe(
      (elements) =>
        (this.form = this.formBuilder.group({
          elements: this.formBuilder.array(elements.map((value) => this.initForm(value)) ?? []),
        })),
    );
  }

  initForm(value: string): FormGroup {
    return this.formBuilder.group({
      value,
      hover: false,
    });
  }

  addElement(): void {
    this.controlElements.push(this.initForm(''));
    this.emitChange();
  }

  removeElement(index: number): void {
    this.controlElements.removeAt(index);
    this.emitChange();
  }

  emitChange(): void {
    this.elementEmitter.emit(this.controlElements.controls.map((control) => control.get('value')?.value));
  }

  drop(event: CdkDragDrop<FormGroup[]>): void {
    moveItemInArray(this.controlElements.controls, event.previousIndex, event.currentIndex);
    this.emitChange();
  }
}
